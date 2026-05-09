pub mod tools;

use anyhow::Context;
use serde_json::Value;
use tracing::Instrument;
use indoc::indoc;

use crate::{
    agent::tools::Tool,
    llm::{
        LlmClient,
        types::{ChatResponse, FunctionDefinition, Message, Role, ToolCall, ToolDefinition},
    },
};

pub struct Agent {
    pub llm: Box<dyn LlmClient>,
    //  TODO: what's the best data structure for tools?
    pub tools: Vec<Box<dyn Tool>>,
    pub max_iterations: usize,
}
// in some utils module, or inline
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...({} more chars)", &s[..max], s.len() - max)
    }
}

/// Render tool call arguments compactly for logs.
///
/// Default `Display` of the raw JSON dumps full string fields (e.g. the entire
/// contents of a `write` call), drowning the line. We pick the keys that
/// matter for navigation and replace large string values with their byte size.
fn summarize_args(tool: &str, args: &Value) -> String {
    let obj = match args.as_object() {
        Some(o) => o,
        None => return args.to_string(),
    };

    let render = |key: &str| -> Option<String> {
        let v = obj.get(key)?;
        Some(match v {
            Value::String(s) if s.len() > 80 => format!("{key}=<{} bytes>", s.len()),
            Value::String(s) => format!("{key}={s:?}"),
            other => format!("{key}={other}"),
        })
    };

    let keys: &[&str] = match tool {
        "read" => &["path"],
        "ls" => &["path"],
        "write" => &["path", "contents"],
        _ => &[],
    };

    if keys.is_empty() {
        return truncate(&args.to_string(), 120);
    }

    keys.iter()
        .filter_map(|k| render(k))
        .collect::<Vec<_>>()
        .join(" ")
}
const SYSTEM_PROMPT: &str = indoc! {"
    You are a coding assistant working in a project directory.

    When using tools:
    - For exploring the project, prefer `ls` and `read` to understand structure before making changes.
    - For modifying existing files, ALWAYS prefer `edit` over `write`. Use `edit` for any change to an existing file, even small ones. Multiple small edits are better than one large write.
    - For creating new files, use `write`.
    - When `edit` fails because of ambiguous or missing matches, add more surrounding context (3-5 lines) and try again.

    Do not guess at file paths. Use `ls` to discover the structure first.
    Do not regenerate file content if you can edit specific sections instead.
"};

impl Agent {
    pub fn new(llm: Box<dyn LlmClient>, tools: Vec<Box<dyn Tool>>) -> Self {
        Self {
            llm,
            tools,
            max_iterations: 100,
        }
    }
    pub async fn run(&self, prompt: String) -> anyhow::Result<String> {
        let mut messages = vec![
            Message {
                role: Role::System,
                content: SYSTEM_PROMPT.into(),
                tool_calls: Option::Some(vec![]),
            },
            Message {
                role: Role::User,
                content: prompt,
                tool_calls: Option::None,
            },
        ];

        let tool_defs: Vec<ToolDefinition> = self
            .tools
            .iter()
            .map(|t| ToolDefinition {
                kind: "function".into(),
                function: FunctionDefinition {
                    name: t.name().into(),
                    description: t.description().into(),
                    parameters: t.parameters_schema(),
                },
            })
            .collect();

        for iteration in 0..self.max_iterations {
            let span = tracing::info_span!("iter", n = iteration);
            let step = async {
                tracing::debug!(
                    last_message_role = ?messages.last().map(|m| &m.role),
                    last_message_preview = %truncate(&messages.last().map(|m| m.content.as_str()).unwrap_or(""), 200),
                    "sending to llm"
                );

                let response: ChatResponse = self
                    .llm
                    .chat(messages.clone(), tool_defs.clone())
                    .await
                    .context("LLM request failed")?;

                let tool_calls = response.message.tool_calls.clone().unwrap_or_default();
                tracing::debug!(
                    role = ?response.message.role,
                    content_preview = %truncate(&response.message.content, 200),
                    tool_call_count = tool_calls.len(),
                    "received llm response"
                );

                if tool_calls.is_empty() {
                    tracing::info!(
                        content_len = response.message.content.len(),
                        "Model returned final answer"
                    );
                    return Ok(Some(response.message.content));
                }

                let names: Vec<&str> = tool_calls
                    .iter()
                    .map(|tc| tc.function.name.as_str())
                    .collect();
                tracing::info!(tools = ?names, "Model requested tool calls");

                messages.push(response.message);
                for tool_call in &tool_calls {
                    messages.push(self.dispatch_tool_call(tool_call).await);
                }
                Ok::<Option<String>, anyhow::Error>(None)
            };

            if let Some(answer) = step.instrument(span).await? {
                return Ok(answer);
            }
        }
        anyhow::bail!("Agent exceeded max iterations")
    }

    #[tracing::instrument(
        skip_all,
        fields(tool = %tool_call.function.name),
    )]
    async fn dispatch_tool_call(&self, tool_call: &ToolCall) -> Message {
        let name = tool_call.function.name.as_str();
        tracing::info!(
            args = %summarize_args(name, &tool_call.function.arguments),
            "tool call",
        );

        let result = match self.tools.iter().find(|tool| tool.name() == name) {
            Some(tool) => tool.execute(tool_call.function.arguments.clone()).await,
            None => Err(anyhow::anyhow!("tool '{name}' not found")),
        };

        let content = match result {
            Ok(out) => {
                tracing::info!(
                    ok = true,
                    bytes = out.len(),
                    preview = %truncate(&out, 160),
                    "tool finished",
                );
                out
            }
            Err(e) => {
                let msg = format!("Error: {e:#}");
                tracing::warn!(ok = false, error = %e, "tool failed");
                msg
            }
        };

        Message {
            role: Role::Tool,
            content,
            tool_calls: Option::None,
        }
    }
}
