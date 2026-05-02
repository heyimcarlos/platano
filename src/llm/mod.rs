pub mod ollama;
pub mod types;

use crate::llm::types::{ChatResponse, Message, ToolDefinition};

use async_trait::async_trait;

#[async_trait]
pub trait LlmClient {
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> anyhow::Result<ChatResponse>;
}
