use serde::Serialize;
use tree_sitter::{Node, Parser, Tree};

use crate::error::{CrawlerError, Result};
use crate::lang::LanguageKind;

#[derive(Debug)]
pub struct ParseOutput {
    pub language_id: String,
    pub parser_id: Option<String>,
    pub parse_success: bool,
    pub syntax_errors_present: bool,
    pub tree: Option<Tree>,
    pub diagnostics: Vec<ParseDiagnostic>,
    pub unsupported_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParseDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
    pub payload_json: serde_json::Value,
}

pub struct ParserRegistry;

impl ParserRegistry {
    pub fn parse(language: LanguageKind, source: &[u8]) -> Result<ParseOutput> {
        match language {
            LanguageKind::UnknownText => Ok(ParseOutput::unsupported(
                language.id(),
                "language classifier returned unknown text",
            )),
            LanguageKind::Toml => parse_toml(source),
            LanguageKind::Yaml => parse_yaml(source),
            LanguageKind::Markdown | LanguageKind::Shell => Ok(ParseOutput {
                language_id: language.id().to_string(),
                parser_id: Some("line-parser@1".to_string()),
                parse_success: true,
                syntax_errors_present: false,
                tree: None,
                diagnostics: Vec::new(),
                unsupported_reason: None,
            }),
            _ => parse_tree_sitter(language, source),
        }
    }
}

impl ParseOutput {
    pub fn unsupported(language_id: &str, reason: &str) -> Self {
        Self {
            language_id: language_id.to_string(),
            parser_id: None,
            parse_success: false,
            syntax_errors_present: false,
            tree: None,
            diagnostics: vec![ParseDiagnostic {
                severity: "info".to_string(),
                code: "unsupported_language".to_string(),
                message: reason.to_string(),
                start_line: None,
                end_line: None,
                payload_json: serde_json::json!({ "reason": reason }),
            }],
            unsupported_reason: Some(reason.to_string()),
        }
    }
}

fn parse_tree_sitter(language: LanguageKind, source: &[u8]) -> Result<ParseOutput> {
    let mut parser = Parser::new();
    let (language_id, parser_id, ts_language) = match language {
        LanguageKind::Rust => (
            "rust",
            "tree-sitter-rust@0.23",
            tree_sitter_rust::LANGUAGE.into(),
        ),
        LanguageKind::TypeScript => (
            "ts",
            "tree-sitter-typescript@0.23",
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        ),
        LanguageKind::Tsx => (
            "tsx",
            "tree-sitter-tsx@0.23",
            tree_sitter_typescript::LANGUAGE_TSX.into(),
        ),
        LanguageKind::JavaScript => (
            "js",
            "tree-sitter-javascript@0.23",
            tree_sitter_javascript::LANGUAGE.into(),
        ),
        LanguageKind::Jsx => (
            "jsx",
            "tree-sitter-javascript@0.23",
            tree_sitter_javascript::LANGUAGE.into(),
        ),
        LanguageKind::Python => (
            "py",
            "tree-sitter-python@0.23",
            tree_sitter_python::LANGUAGE.into(),
        ),
        LanguageKind::Json => return parse_json(source),
        _ => {
            return Ok(ParseOutput::unsupported(
                language.id(),
                "no parser registered for language",
            ))
        }
    };

    parser
        .set_language(&ts_language)
        .map_err(|error| CrawlerError::Config(format!("tree-sitter language error: {error}")))?;
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| CrawlerError::Config("tree-sitter parser returned no tree".to_string()))?;
    let root = tree.root_node();
    let syntax_errors_present = root.has_error();
    let mut diagnostics = Vec::new();
    if syntax_errors_present {
        collect_tree_sitter_errors(root, &mut diagnostics);
        if diagnostics.is_empty() {
            diagnostics.push(ParseDiagnostic {
                severity: "error".to_string(),
                code: "syntax_error".to_string(),
                message: "syntax errors present in parse tree".to_string(),
                start_line: Some(1),
                end_line: Some(1),
                payload_json: serde_json::json!({ "parser_id": parser_id }),
            });
        }
    }

    Ok(ParseOutput {
        language_id: language_id.to_string(),
        parser_id: Some(parser_id.to_string()),
        parse_success: !syntax_errors_present,
        syntax_errors_present,
        tree: Some(tree),
        diagnostics,
        unsupported_reason: None,
    })
}

fn parse_json(source: &[u8]) -> Result<ParseOutput> {
    match serde_json::from_slice::<serde_json::Value>(source) {
        Ok(_) => Ok(ParseOutput {
            language_id: "json".to_string(),
            parser_id: Some("serde-json@1".to_string()),
            parse_success: true,
            syntax_errors_present: false,
            tree: None,
            diagnostics: Vec::new(),
            unsupported_reason: None,
        }),
        Err(error) => Ok(ParseOutput {
            language_id: "json".to_string(),
            parser_id: Some("serde-json@1".to_string()),
            parse_success: false,
            syntax_errors_present: true,
            tree: None,
            diagnostics: vec![ParseDiagnostic {
                severity: "error".to_string(),
                code: "json_parse_error".to_string(),
                message: error.to_string(),
                start_line: Some(error.line() as u32),
                end_line: Some(error.line() as u32),
                payload_json: serde_json::json!({ "parser": "serde_json" }),
            }],
            unsupported_reason: None,
        }),
    }
}

fn parse_toml(source: &[u8]) -> Result<ParseOutput> {
    let text = String::from_utf8_lossy(source);
    match text.parse::<toml::Value>() {
        Ok(_) => Ok(ParseOutput {
            language_id: "toml".to_string(),
            parser_id: Some("toml@0.8".to_string()),
            parse_success: true,
            syntax_errors_present: false,
            tree: None,
            diagnostics: Vec::new(),
            unsupported_reason: None,
        }),
        Err(error) => Ok(ParseOutput {
            language_id: "toml".to_string(),
            parser_id: Some("toml@0.8".to_string()),
            parse_success: false,
            syntax_errors_present: true,
            tree: None,
            diagnostics: vec![ParseDiagnostic {
                severity: "error".to_string(),
                code: "toml_parse_error".to_string(),
                message: error.message().to_string(),
                start_line: None,
                end_line: None,
                payload_json: serde_json::json!({ "parser": "toml" }),
            }],
            unsupported_reason: None,
        }),
    }
}

fn parse_yaml(source: &[u8]) -> Result<ParseOutput> {
    match serde_yaml::from_slice::<serde_yaml::Value>(source) {
        Ok(_) => Ok(ParseOutput {
            language_id: "yaml".to_string(),
            parser_id: Some("serde-yaml@0.9".to_string()),
            parse_success: true,
            syntax_errors_present: false,
            tree: None,
            diagnostics: Vec::new(),
            unsupported_reason: None,
        }),
        Err(error) => Ok(ParseOutput {
            language_id: "yaml".to_string(),
            parser_id: Some("serde-yaml@0.9".to_string()),
            parse_success: false,
            syntax_errors_present: true,
            tree: None,
            diagnostics: vec![ParseDiagnostic {
                severity: "error".to_string(),
                code: "yaml_parse_error".to_string(),
                message: error.to_string(),
                start_line: error.location().map(|location| location.line() as u32),
                end_line: error.location().map(|location| location.line() as u32),
                payload_json: serde_json::json!({ "parser": "serde_yaml" }),
            }],
            unsupported_reason: None,
        }),
    }
}

fn collect_tree_sitter_errors(node: Node<'_>, diagnostics: &mut Vec<ParseDiagnostic>) {
    if node.is_error() || node.is_missing() {
        diagnostics.push(ParseDiagnostic {
            severity: "error".to_string(),
            code: if node.is_missing() {
                "missing_node".to_string()
            } else {
                "syntax_error".to_string()
            },
            message: format!("tree-sitter reported {}", node.kind()),
            start_line: Some((node.start_position().row + 1) as u32),
            end_line: Some((node.end_position().row + 1) as u32),
            payload_json: serde_json::json!({
                "kind": node.kind(),
                "start_byte": node.start_byte(),
                "end_byte": node.end_byte()
            }),
        });
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_tree_sitter_errors(child, diagnostics);
    }
}
