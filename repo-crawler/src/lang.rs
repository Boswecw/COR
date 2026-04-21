use std::path::Path;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LanguageKind {
    Rust,
    TypeScript,
    Tsx,
    JavaScript,
    Jsx,
    Json,
    Toml,
    Yaml,
    Markdown,
    Python,
    Shell,
    UnknownText,
}

impl LanguageKind {
    pub fn id(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "ts",
            Self::Tsx => "tsx",
            Self::JavaScript => "js",
            Self::Jsx => "jsx",
            Self::Json => "json",
            Self::Toml => "toml",
            Self::Yaml => "yaml",
            Self::Markdown => "md",
            Self::Python => "py",
            Self::Shell => "sh",
            Self::UnknownText => "unknown",
        }
    }
}

pub fn classify_path(path: &Path) -> LanguageKind {
    let filename = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match filename.as_str() {
        "cargo.toml" | "pyproject.toml" | "taplo.toml" => return LanguageKind::Toml,
        "makefile" | "bashrc" | "zshrc" => return LanguageKind::Shell,
        _ => {}
    }

    match extension.as_str() {
        "rs" => LanguageKind::Rust,
        "ts" | "mts" | "cts" => LanguageKind::TypeScript,
        "tsx" => LanguageKind::Tsx,
        "js" | "mjs" | "cjs" => LanguageKind::JavaScript,
        "jsx" => LanguageKind::Jsx,
        "json" => LanguageKind::Json,
        "toml" => LanguageKind::Toml,
        "yaml" | "yml" => LanguageKind::Yaml,
        "md" | "markdown" => LanguageKind::Markdown,
        "py" | "pyw" => LanguageKind::Python,
        "sh" | "bash" | "zsh" | "fish" => LanguageKind::Shell,
        _ => LanguageKind::UnknownText,
    }
}
