use crate::{config::Settings, llm::LlmClient};

pub struct App {
    pub llm: LlmClient,
}

pub fn build(settings: Settings) -> anyhow::Result<App> {
    let llm = LlmClient::new(settings.llm.base_url, settings.llm.model);
    Ok(App { llm })
}
