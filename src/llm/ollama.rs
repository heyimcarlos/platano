use async_trait::async_trait;
use reqwest::Client;
use tracing::instrument;

use super::{LlmClient, types::*};

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
    #[instrument(skip(self, messages), fields(model = %self.model, msg_count = messages.len()))]
    async fn chat(&self, messages: Vec<Message>) -> anyhow::Result<ChatResponse> {
        let url = format!("{}/api/chat", self.base_url);
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            options: ChatOptions {
                num_ctx: 8192,
                temperature: 0.7,
            },
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
