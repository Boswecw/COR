use std::collections::HashSet;
use std::path::{Component, Path};

use serde::Serialize;

use crate::config::RepoCrawlerConfig;

#[derive(Debug, Clone, Serialize)]
pub struct SkippedPath {
    pub rel_path: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct Policy {
    exclude_dirs: HashSet<String>,
    exclude_extensions: HashSet<String>,
    include_binary: bool,
    include_generated_lockfiles: bool,
    max_file_size_bytes: u64,
}

impl Policy {
    pub fn new(config: &RepoCrawlerConfig) -> Self {
        Self {
            exclude_dirs: config
                .crawl
                .exclude_dirs
                .iter()
                .map(|value| value.to_ascii_lowercase())
                .collect(),
            exclude_extensions: config
                .crawl
                .exclude_extensions
                .iter()
                .map(|value| value.trim_start_matches('.').to_ascii_lowercase())
                .collect(),
            include_binary: config.crawl.include_binary,
            include_generated_lockfiles: config.crawl.include_generated_lockfiles,
            max_file_size_bytes: config.crawl.max_file_size_bytes,
        }
    }

    pub fn skip_dir_reason(&self, rel_path: &Path) -> Option<String> {
        for component in rel_path.components() {
            if let Component::Normal(value) = component {
                let name = value.to_string_lossy().to_ascii_lowercase();
                if self.exclude_dirs.contains(&name) {
                    return Some(format!("excluded_dir:{name}"));
                }
            }
        }
        None
    }

    pub fn skip_file_reason(
        &self,
        rel_path: &Path,
        size_bytes: u64,
        is_binary: bool,
    ) -> Option<String> {
        if let Some(reason) = self.skip_dir_reason(rel_path) {
            return Some(reason);
        }

        if size_bytes > self.max_file_size_bytes {
            return Some(format!("size_limit:>{}", self.max_file_size_bytes));
        }

        if !self.include_generated_lockfiles && is_generated_lockfile(rel_path) {
            return Some("generated_lockfile".to_string());
        }

        let extension = rel_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .trim_start_matches('.')
            .to_ascii_lowercase();
        if !extension.is_empty() && self.exclude_extensions.contains(&extension) {
            return Some(format!("excluded_extension:{extension}"));
        }

        if is_binary && !self.include_binary {
            return Some("binary_denied".to_string());
        }

        None
    }
}

fn is_generated_lockfile(path: &Path) -> bool {
    matches!(
        path.file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default(),
        "Cargo.lock"
            | "package-lock.json"
            | "yarn.lock"
            | "pnpm-lock.yaml"
            | "bun.lockb"
            | "poetry.lock"
            | "Pipfile.lock"
            | "Gemfile.lock"
            | "composer.lock"
    )
}
