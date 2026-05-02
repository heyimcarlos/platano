pub mod tools;

use anyhow::Context;

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

impl Agent {
    pub fn new(llm: Box<dyn LlmClient>, tools: Vec<Box<dyn Tool>>) -> Self {
        Self {
            llm,
            tools,
            max_iterations: 10,
        }
    }
    pub async fn run(&self, prompt: String) -> anyhow::Result<String> {
        let mut messages = vec![
            Message {
                role: Role::System,
                content: "Your are a helpful assistant with access to tools. Use them whenever \
                     the user asks about file contents or anything that requires reading \
                     the local filesystem. Do not guess file contents"
                    .into(),
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
            tracing::debug!(
                iteration,
                last_message_role = ?messages.last().map(|m| &m.role),
                last_message_preview = %truncate(&messages.last().map(|m| m.content.as_str()).unwrap_or(""), 200),
                "sending to llm"
            );

            let response: ChatResponse = self
                .llm
                .chat(messages.clone(), tool_defs.clone())
                .await
                .context("LLM request failed")?;

            tracing::debug!(
                role = ?response.message.role,
                content_preview = %truncate(&response.message.content, 200),
                tool_call_count = response.message.tool_calls.as_ref().map(|t| t.len()).unwrap_or(0),
                "received llm response"
            );

            let tool_calls = response.message.tool_calls.clone().unwrap_or_default();
            if tool_calls.is_empty() {
                tracing::info!(
                    content_len = response.message.content.len(),
                    "Model returned final answer"
                );
                return Ok(response.message.content);
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
        }
        anyhow::bail!("Agent exceeded max iterations")
    }

    async fn dispatch_tool_call(&self, tool_call: &ToolCall) -> Message {
        tracing::debug!(
            tool = %tool_call.function.name,
            args = %tool_call.function.arguments,
            "dispatching tool call"
        );
        let content = match self
            .tools
            .iter()
            .find(|tool| tool.name() == tool_call.function.name)
        {
            Some(tool) => match tool.execute(tool_call.function.arguments.clone()).await {
                Ok(res) => res,
                Err(e) => format!("Error: {e:#}"),
            },
            None => format!("Error: tool '{}' not found", tool_call.function.name),
        };

        Message {
            role: Role::Tool,
            content,
            tool_calls: Option::None,
        }
    }
}
