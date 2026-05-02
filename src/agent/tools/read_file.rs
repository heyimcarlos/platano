use anyhow::{Context, anyhow};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use tracing::instrument;

use crate::agent::tools::Tool;

const MAX_FILE_SIZE: u64 = 1024 * 1024; // 1 MB

pub struct ReadFile {}

impl ReadFile {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Deserialize)]
struct ReadFileArgs {
    path: String,
}

#[async_trait]
impl Tool for ReadFile {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Reads the contents of a text file from the project directory. \
        Returns the file's contents as a string. Only use for text files."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path to the file from the project root."
                }
            },
            "required": ["path"]
        })
    }

    #[instrument(skip_all, fields(tool = %self.name()), level = "debug")]
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<String> {
        let args: ReadFileArgs =
            serde_json::from_value(args).context("Invalid arguments for read_file")?;

        let metadata = tokio::fs::metadata(&args.path)
            .await
            .with_context(|| format!("Failed to stat {}", args.path))?;

        if !metadata.is_file() {
            return Err(anyhow!("Path is not a file: {}", args.path));
        }

        if metadata.len() > MAX_FILE_SIZE {
            return Err(anyhow!(
                "File is too large ({} bytes, max {} bytes)",
                metadata.len(),
                MAX_FILE_SIZE
            ));
        }

        let contents = tokio::fs::read_to_string(&args.path)
            .await
            .with_context(|| format!("Failed to read {}", args.path))?;

        tracing::info!("Read file {} ({} bytes)", args.path, contents.len());
        Ok(contents)
    }
}
