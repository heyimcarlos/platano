use std::path::PathBuf;

use anyhow::Context;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use crate::agent::tools::{Tool, path::resolve_safe_path};

const MAX_WRITE_SIZE: usize = 1024 * 1024;

pub struct WriteFile {
    root: PathBuf,
}

impl WriteFile {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

#[derive(Deserialize)]
struct WriteFileArgs {
    path: String,
    contents: String,
}

#[async_trait]
impl Tool for WriteFile {
    fn name(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        "Writes content to a file in the project directory. Creates the file \
        if it does not exist, overwrites if it does. Use for creating new \
        files or fully replacing existing ones. For partial edits, use `edit`."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path to the file from the project root."
                },
                "contents": {
                    "type": "string",
                    "description": "Contents to write to the file."
                }
            },
            "required": ["path", "contents"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<String> {
        let args: WriteFileArgs =
            serde_json::from_value(args).context("Invalid argument for write")?;

        if args.contents.len() > MAX_WRITE_SIZE {
            anyhow::bail!(
                "Contents too large ({} bytes, max {} bytes)",
                args.contents.len(),
                MAX_WRITE_SIZE
            );
        }

        let resolved = resolve_safe_path(&self.root, &args.path)?;

        tokio::fs::write(&resolved, &args.contents)
            .await
            .with_context(|| format!("Failed to write {}", resolved.display()))?;

        Ok(format!(
            "Wrote {} bytes to {}",
            args.contents.len(),
            args.path
        ))
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};
    use serde_json::json;
    use tempfile::TempDir;

    use crate::agent::tools::{Tool, write_file::WriteFile};

    #[tokio::test]
    async fn write_new_file() {
        let dir = TempDir::new().unwrap();
        let tool = WriteFile::new(dir.path().to_path_buf());

        let result = tool
            .execute(json!({
                "path": "hello.txt",
                "contents": "world"
            }))
            .await;

        assert_ok!(result);
        let contents = std::fs::read_to_string(dir.path().join("hello.txt")).unwrap();
        tracing::info!("File contents: {}", contents);
        assert!(contents.contains("world"));
    }

    #[tokio::test]
    async fn overwrite_existing_file() {
        let dir = TempDir::new().unwrap();
        let tool = WriteFile::new(dir.path().to_path_buf());

        tool.execute(json!({
            "path": "hello.txt",
            "contents": "world"
        }))
        .await
        .unwrap();

        let result = tool
            .execute(json!({
                "path": "hello.txt",
                "contents": "new contents"
            }))
            .await;

        assert_ok!(result);
        let contents = std::fs::read_to_string(dir.path().join("hello.txt")).unwrap();
        assert!(contents.contains("new contents"));
    }

    #[tokio::test]
    async fn rejects_path_outside_sandbox() {
        let dir = TempDir::new().unwrap();
        let tool = WriteFile::new(dir.path().to_path_buf());

        let result = tool
            .execute(json!({
                "path": "../../hacked.txt",
                "contents": "hacked"
            }))
            .await;

        assert_err!(result);
    }

    #[tokio::test]
    async fn rejects_oversized_content() {
        let dir = TempDir::new().unwrap();
        let tool = WriteFile::new(dir.path().to_path_buf());

        let big = "x".repeat(2 * 1024 * 1024);
        let result = tool
            .execute(json!({
                "path": "big.txt",
                "contents": big
            }))
            .await;

        assert_err!(result);
    }
}
