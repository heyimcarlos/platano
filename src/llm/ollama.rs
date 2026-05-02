use anyhow::Context;
use reqwest::Client;
use tracing::instrument;

use super::{LlmClient, types::*};

use async_trait::async_trait;

pub struct OllamaClient {
    http: Client,
    base_url: String,
    model: String,
}

impl OllamaClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            http: Client::new(),
            base_url,
            model,
        }
    }
}

#[async_trait]
impl LlmClient for OllamaClient {
    #[instrument(skip_all, fields(model = %self.model, msg_count = messages.len()), level = "debug")]
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> anyhow::Result<ChatResponse> {
        let url = format!("{}/api/chat", self.base_url);
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            options: ChatOptions {
                num_ctx: 8192,
                temperature: 0.7,
            },
            tools,
        };

        let response = self
            .http
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request")?
            .error_for_status()
            .context("Ollama returned error")?
            .json::<ChatResponse>()
            .await
            .context("Failed to parse response")?;

        Ok(response)
    }
}
