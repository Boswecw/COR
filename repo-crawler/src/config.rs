use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{CrawlerError, Result};

pub const CONFIG_VERSION: u32 = 1;
pub const DEFAULT_CONFIG_REL_PATH: &str = ".repo-crawler/config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RepoCrawlerConfig {
    pub version: u32,
    pub repo: RepoSection,
    pub crawl: CrawlSection,
    pub parse: ParseSection,
    pub watch: WatchSection,
    pub store: StoreSection,
    pub export: ExportSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RepoSection {
    pub root: String,
    pub follow_symlinks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CrawlSection {
    pub respect_gitignore: bool,
    pub include_hidden: bool,
    pub max_file_size_bytes: u64,
    pub exclude_dirs: Vec<String>,
    pub exclude_extensions: Vec<String>,
    pub include_binary: bool,
    pub include_generated_lockfiles: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ParseSection {
    pub enabled_languages: Vec<String>,
    pub max_parser_workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WatchSection {
    pub enabled: bool,
    pub debounce_ms: u64,
    pub poll_reconcile_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StoreSection {
    pub sqlite_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportSection {
    pub default_format: String,
    pub include_diagnostics: bool,
}

impl Default for RepoCrawlerConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            repo: RepoSection::default(),
            crawl: CrawlSection::default(),
            parse: ParseSection::default(),
            watch: WatchSection::default(),
            store: StoreSection::default(),
            export: ExportSection::default(),
        }
    }
}

impl Default for RepoSection {
    fn default() -> Self {
        Self {
            root: ".".to_string(),
            follow_symlinks: false,
        }
    }
}

impl Default for CrawlSection {
    fn default() -> Self {
        Self {
            respect_gitignore: true,
            include_hidden: false,
            max_file_size_bytes: 1_048_576,
            exclude_dirs: vec![
                ".git".to_string(),
                ".repo-crawler".to_string(),
                "node_modules".to_string(),
                "dist".to_string(),
                "build".to_string(),
                "target".to_string(),
                "coverage".to_string(),
            ],
            exclude_extensions: vec![
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "gif".to_string(),
                "pdf".to_string(),
                "zip".to_string(),
                "tar".to_string(),
                "gz".to_string(),
            ],
            include_binary: false,
            include_generated_lockfiles: false,
        }
    }
}

impl Default for ParseSection {
    fn default() -> Self {
        Self {
            enabled_languages: vec![
                "rust".to_string(),
                "ts".to_string(),
                "tsx".to_string(),
                "js".to_string(),
                "jsx".to_string(),
                "py".to_string(),
                "json".to_string(),
                "toml".to_string(),
                "yaml".to_string(),
                "md".to_string(),
                "sh".to_string(),
            ],
            max_parser_workers: 8,
        }
    }
}

impl Default for WatchSection {
    fn default() -> Self {
        Self {
            enabled: false,
            debounce_ms: 500,
            poll_reconcile_seconds: 30,
        }
    }
}

impl Default for StoreSection {
    fn default() -> Self {
        Self {
            sqlite_path: ".repo-crawler/index.db".to_string(),
        }
    }
}

impl Default for ExportSection {
    fn default() -> Self {
        Self {
            default_format: "json".to_string(),
            include_diagnostics: true,
        }
    }
}

impl RepoCrawlerConfig {
    pub fn load(root: &Path, config_path: Option<&Path>) -> Result<Self> {
        let candidate = match config_path {
            Some(path) => path.to_path_buf(),
            None => root.join(DEFAULT_CONFIG_REL_PATH),
        };

        if !candidate.exists() {
            return Ok(Self::default());
        }

        let raw = fs::read_to_string(candidate)?;
        let config: Self = toml::from_str(&raw)?;
        if config.version != CONFIG_VERSION {
            return Err(CrawlerError::Config(format!(
                "unsupported config version {}; expected {CONFIG_VERSION}",
                config.version
            )));
        }
        Ok(config)
    }

    pub fn write_default(root: &Path, force: bool) -> Result<PathBuf> {
        let config_path = root.join(DEFAULT_CONFIG_REL_PATH);
        if config_path.exists() && !force {
            return Err(CrawlerError::Config(format!(
                "{} already exists; pass --force to overwrite",
                config_path.display()
            )));
        }
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let body = toml::to_string_pretty(&Self::default())?;
        fs::write(&config_path, body)?;
        Ok(config_path)
    }

    pub fn store_path(&self, root: &Path) -> PathBuf {
        let configured = PathBuf::from(&self.store.sqlite_path);
        if configured.is_absolute() {
            configured
        } else {
            root.join(configured)
        }
    }

    pub fn language_enabled(&self, language_id: &str) -> bool {
        self.parse
            .enabled_languages
            .iter()
            .any(|enabled| enabled == language_id)
    }
}
