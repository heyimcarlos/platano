use anyhow::{Context, anyhow};
use std::path::{Path, PathBuf};

//  TODO: move to a struct Sandbox {root: PathBuf}
//  impl Sandbox {pub fn resolve(&self, requested: &str)}

/// Resolve a user provided path against a sandbox root.
pub(crate) fn resolve_safe_path(root: &Path, requested: &str) -> anyhow::Result<PathBuf> {
    let canonical_root = root
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize root {}", root.display()))?;

    let joined = canonical_root.join(requested);

    let canonical_target = if joined.exists() {
        joined
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize target {}", joined.display()))?
    } else {
        let parent = joined
            .parent()
            .ok_or_else(|| anyhow!("Path has no parent: {}", joined.display()))?;

        let file_name = joined
            .file_name()
            .ok_or_else(|| anyhow!("Path has no filename: {}", joined.display()))?;

        let canonical_parent = parent
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize parent {}", parent.display()))?;

        canonical_parent.join(file_name)
    };

    if !canonical_target.starts_with(&canonical_root) {
        return Err(anyhow!(
            "Path escapes project root: {}",
            canonical_target.display()
        ));
    }

    Ok(canonical_target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn resolves_path_inside_root() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("hello.txt"), "").unwrap();

        let resolved = resolve_safe_path(dir.path(), "hello.txt").unwrap();
        assert!(resolved.starts_with(dir.path().canonicalize().unwrap()));
    }

    #[test]
    fn rejects_parent_escape() {
        let dir = TempDir::new().unwrap();
        let result = resolve_safe_path(dir.path(), "../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn rejects_absolute_path_escape() {
        let dir = TempDir::new().unwrap();
        let result = resolve_safe_path(dir.path(), "/etc/passwd");
        assert!(result.is_err());
    }
}
