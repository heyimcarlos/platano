use crate::{
    config::Settings,
    llm::{LlmClient, ollama::OllamaClient},
};

pub struct App {
    pub llm: Box<dyn LlmClient>,
}

pub fn build(settings: Settings) -> anyhow::Result<App> {
    let llm: Box<dyn LlmClient> =
        Box::new(OllamaClient::new(settings.llm.base_url, settings.llm.model));
    Ok(App { llm })
}
