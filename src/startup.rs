use crate::{
    agent::{
        Agent,
        tools::{Tool, read_file::ReadFile},
    },
    config::Settings,
    llm::{LlmClient, ollama::OllamaClient},
};

pub fn build(settings: Settings) -> anyhow::Result<Agent> {
    let llm: Box<dyn LlmClient> =
        Box::new(OllamaClient::new(settings.llm.base_url, settings.llm.model));

    let mut tools: Vec<Box<dyn Tool>> = vec![];
    let read_file = Box::new(ReadFile::new());
    tools.push(read_file);
    Ok(Agent::new(llm, tools))
}
