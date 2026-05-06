use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Tool: Send + Sync {
    /// The name the llm uses to call the toop.
    fn name(&self) -> &str;

    /// Human/LLM-facing description of what the tool does.
    fn description(&self) -> &str;

    /// JSON schema for the tool's parameters.
    fn parameters_schema(&self) -> Value;

    /// Execute the tool with deserialized arguments.
    async fn execute(&self, args: Value) -> anyhow::Result<String>;
}

pub mod list_directory;
pub mod read_file;
pub mod search_files;
