use std::path::PathBuf;

use anyhow::{Context, anyhow};
use async_trait::async_trait;
use serde_json::json;

use crate::agent::tools::{Tool, path::resolve_safe_path};

pub struct EditFile {
    root: PathBuf,
}

impl EditFile {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

#[derive(serde::Deserialize)]
struct EditFileArgs {
    path: String,
    old_str: String,
    new_str: String,
}

#[async_trait]
impl Tool for EditFile {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        "Edits an existing file by replacing exact text. **This is the preferred \
        tool for any change to an existing file**, including small fixes, additions, \
        removals, and refactors. Provide `old_str` (existing text) and `new_str` \
        (replacement). The `old_str` must match exactly once include 3-5 lines of \
        surrounding context. Use multiple edit calls for multiple changes; do not \
        use write to combine them."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path to the file from the project root."
                },
                "old_str": {
                    "type": "string",
                    "description": "Exact text to find in the file. Must appear exactly once. Include surrounding context to disambiguate."
                },
                "new_str": {
                    "type": "string",
                    "description": "Text to replace old_str with."
                }
            },
            "required": ["path", "old_str", "new_str"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<String> {
        let args: EditFileArgs =
            serde_json::from_value(args).context("Invalid argument to edit")?;

        let resolved = resolve_safe_path(&self.root, &args.path)?;

        let metadata = tokio::fs::metadata(&resolved)
            .await
            .with_context(|| format!("Failed to stat {}", resolved.display()))?;

        if !metadata.is_file() {
            return Err(anyhow!("Failed to stat {}", resolved.display()));
        }

        let contents = tokio::fs::read_to_string(&resolved)
            .await
            .with_context(|| format!("Failed to read {}", resolved.display()))?;

        let match_count = contents.matches(&args.old_str).count();

        if match_count == 0 {
            return Err(anyhow!(
                "old_str not found in {}. The text must match exactly, including whitespace.",
                args.path
            ));
        }

        if match_count > 1 {
            return Err(anyhow!(
                "old_str matched {} time in {}. It must match exactly once. \
            Include more surrounding context (3-5 lines) to make the match unique.",
                match_count,
                args.path
            ));
        }

        let new_contents = contents.replacen(&args.old_str, &args.new_str, 1);

        tokio::fs::write(&resolved, &new_contents)
            .await
            .with_context(|| format!("Failed to write {}", args.path))?;

        let bytes_delta = new_contents.len() - contents.len();

        Ok(format!(
            "Edited {}. Replaced {} bytes with {} bytes (delta: {:+}).",
            args.path,
            args.old_str.len(),
            args.new_str.len(),
            bytes_delta
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};
    use serde_json::json;
    use tempfile::TempDir;

    #[tokio::test]
    async fn edits_unique_match() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("foo.txt");
        std::fs::write(&path, "hello world\nfoo bar\nbaz").unwrap();
        let tool = EditFile::new(dir.path().to_path_buf());

        let result = tool
            .execute(json!({
                "path": "foo.txt",
                "old_str": "hello world",
                "new_str": "goodbye world"
            }))
            .await;

        assert_ok!(result);
        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "goodbye world\nfoo bar\nbaz");
    }

    #[tokio::test]
    async fn rejects_no_match() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("foo.txt"), "hello").unwrap();
        let tool = EditFile::new(dir.path().to_path_buf());

        let result = tool
            .execute(json!({
                "path": "foo.txt",
                "old_str": "not present",
                "new_str": "x"
            }))
            .await;

        assert_err!(result);
    }

    #[tokio::test]
    async fn rejects_multiple_matches() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("foo.txt"), "x\nx\nx").unwrap();
        let tool = EditFile::new(dir.path().to_path_buf());

        let result = tool
            .execute(json!({
                "path": "foo.txt",
                "old_str": "x",
                "new_str": "y"
            }))
            .await;

        assert_err!(result);
    }

    #[tokio::test]
    async fn rejects_path_outside_sandbox() {
        let dir = TempDir::new().unwrap();
        let tool = EditFile::new(dir.path().to_path_buf());

        let result = tool
            .execute(json!({
                "path": "../../etc/passwd",
                "old_str": "x",
                "new_str": "y"
            }))
            .await;

        assert_err!(result);
    }

    #[tokio::test]
    async fn rejects_directory() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();
        let tool = EditFile::new(dir.path().to_path_buf());

        let result = tool
            .execute(json!({
                "path": "subdir",
                "old_str": "x",
                "new_str": "y"
            }))
            .await;

        assert_err!(result);
    }
}
