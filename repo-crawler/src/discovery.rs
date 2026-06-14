use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

use crate::error::{CrawlerError, Result};

#[derive(Debug, Clone, Serialize)]
pub struct RepoIdentity {
    pub root_path: PathBuf,
    pub vcs_type: Option<String>,
    pub head_ref: Option<String>,
    pub head_commit: Option<String>,
}

pub fn discover_repo_root(input: &Path) -> Result<PathBuf> {
    let canonical = input
        .canonicalize()
        .map_err(|error| CrawlerError::InvalidRoot(format!("{} ({error})", input.display())))?;
    let start = if canonical.is_file() {
        canonical.parent().ok_or_else(|| {
            CrawlerError::InvalidRoot(format!("{} has no parent", canonical.display()))
        })?
    } else {
        canonical.as_path()
    };

    if let Some(git_root) = find_git_root(start) {
        Ok(git_root)
    } else if start.is_dir() {
        Ok(start.to_path_buf())
    } else {
        Err(CrawlerError::InvalidRoot(format!(
            "{} is not a directory",
            start.display()
        )))
    }
}

pub fn discover_repo(input: &Path) -> Result<RepoIdentity> {
    let root_path = discover_repo_root(input)?;
    let (head_ref, head_commit) = git_metadata(&root_path)?;
    let vcs_type = if root_path.join(".git").exists() {
        Some("git".to_string())
    } else {
        None
    };
    Ok(RepoIdentity {
        root_path,
        vcs_type,
        head_ref,
        head_commit,
    })
}

fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut cursor = Some(start);
    while let Some(path) = cursor {
        if path.join(".git").exists() {
            return Some(path.to_path_buf());
        }
        cursor = path.parent();
    }
    None
}

fn git_metadata(root: &Path) -> Result<(Option<String>, Option<String>)> {
    if !root.join(".git").exists() {
        return Ok((None, None));
    }

    let head_ref = run_git(root, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let head_commit = run_git(root, &["rev-parse", "HEAD"])?;
    Ok((head_ref, head_commit))
}

pub fn staged_paths(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.join(".git").exists() {
        return Err(CrawlerError::Git(format!(
            "{} is not a git repository",
            root.display()
        )));
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["diff", "--name-only", "--cached"])
        .output()?;
    if !output.status.success() {
        return Err(CrawlerError::Git(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(PathBuf::from)
        .collect())
}

fn run_git(root: &Path, args: &[&str]) -> Result<Option<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()?;
    if !output.status.success() {
        return Ok(None);
    }
    let value = String::from_utf8(output.stdout)?.trim().to_string();
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}
