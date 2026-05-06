use crate::{
    agent::{
        Agent,
        tools::{Tool, list_directory::ListDirectory, read_file::ReadFile, write_file::WriteFile},
    },
    config::Settings,
    llm::{LlmClient, ollama::OllamaClient},
};

pub fn build(settings: Settings) -> anyhow::Result<Agent> {
    let llm: Box<dyn LlmClient> =
        Box::new(OllamaClient::new(settings.llm.base_url, settings.llm.model));

    let cwd = std::env::current_dir()?;

    let read_file = Box::new(ReadFile::new(cwd.clone()));
    let write_file = Box::new(WriteFile::new(cwd.clone()));
    let list_directory = Box::new(ListDirectory::new(cwd.clone()));
    let tools: Vec<Box<dyn Tool>> = vec![read_file, write_file, list_directory];
    Ok(Agent::new(llm, tools))
}
