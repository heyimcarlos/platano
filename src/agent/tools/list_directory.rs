use std::path::PathBuf;

use anyhow::{Context, anyhow};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use tracing::instrument;

use crate::agent::tools::Tool;

pub struct ListDirectory {
    root: PathBuf,
}

impl ListDirectory {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

#[derive(Deserialize)]
struct ListDirectoryArgs {
    path: String,
}

#[async_trait]
impl Tool for ListDirectory {
    fn name(&self) -> &str {
        "list_directory"
    }

    fn description(&self) -> &str {
        "Lists files within the current directory. \
        Returns a list of all files in the current working directory."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path to the directory to list."
                }
            },
            "required": ["path"]
        })
    }

    #[instrument(skip_all, fields(tool = &self.name()), level = "debug")]
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<String> {
        let args: ListDirectoryArgs =
            serde_json::from_value(args).context("Invalid arguments for list_directory")?;

        let metadata = tokio::fs::metadata(&args.path)
            .await
            .with_context(|| format!("Failed to stat {}", args.path))?;

        if !metadata.is_dir() {
            return Err(anyhow!("Path is not a directory: {}", args.path));
        }

        // let metadata = tokio::fs::canonicalize()
        let mut entries = tokio::fs::read_dir(&args.path)
            .await
            .with_context(|| format!("Failed to read {}", args.path))?;

        //  INFO:Non-rusty approach
        // let mut result = format!("Contents of {}\n", &args.path);
        // while let Some(entry) = entries.next_entry().await? {
        //     let mut line = entry.file_name().to_string_lossy().into_owned();
        //     if let Ok(file_type) = entry.file_type().await {
        //         if file_type.is_dir() {
        //             line.push('/')
        //         }
        //     }
        //     result.push_str(&line);
        //     result.push('\n');
        // }

        let mut entries_vec = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().to_string_lossy().into_owned();
            let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
            entries_vec.push(if is_dir { format!("{name}/") } else { name });
        }
        entries_vec.sort();

        let mut result = format!("Contents of {}:\n", args.path);
        result.push_str(&entries_vec.join("\n"));

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::agent::tools::{Tool, list_directory::ListDirectory};
    use serde_json::json;
    use tempfile::TempDir;

    #[tokio::test]
    async fn list_files_in_a_directory() {
        // Arrange
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.txt"), "").unwrap();
        std::fs::write(dir.path().join("b.txt"), "").unwrap();

        let tool = ListDirectory::new(dir.path().to_path_buf());

        // Act
        let result = tool.execute(json!({"path": dir.path()})).await.unwrap();

        // Assert
        assert!(result.contains("b.txt"))
    }
}
