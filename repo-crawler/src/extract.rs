use serde::Serialize;
use tree_sitter::{Node, Tree};

use crate::lang::LanguageKind;

#[derive(Debug, Clone, Serialize)]
pub struct SymbolFact {
    pub kind: String,
    pub name: String,
    pub qualname: String,
    pub start_byte: u64,
    pub end_byte: u64,
    pub start_line: u32,
    pub end_line: u32,
    pub visibility: Option<String>,
    pub signature: Option<String>,
    pub doc_excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EdgeFact {
    pub edge_kind: String,
    pub payload_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileMetrics {
    pub total_lines: u32,
    pub blank_lines: u32,
    pub comment_lines: u32,
    pub todo_count: u32,
    pub fixme_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtractionFacts {
    pub symbols: Vec<SymbolFact>,
    pub edges: Vec<EdgeFact>,
    pub metrics: FileMetrics,
}

#[derive(Debug)]
struct LineIndex {
    starts: Vec<usize>,
}

pub fn extract(language: LanguageKind, source: &[u8], tree: Option<&Tree>) -> ExtractionFacts {
    let text = String::from_utf8_lossy(source);
    let line_index = LineIndex::new(&text);
    let mut facts = ExtractionFacts {
        symbols: Vec::new(),
        edges: Vec::new(),
        metrics: compute_metrics(language, &text),
    };

    if let Some(tree) = tree {
        visit_tree(language, tree.root_node(), source, &line_index, &mut facts);
    }

    match language {
        LanguageKind::Markdown => extract_markdown_headings(&text, &line_index, &mut facts),
        LanguageKind::Shell => extract_shell_symbols_and_edges(&text, &line_index, &mut facts),
        _ => {}
    }

    extract_markers(&text, &line_index, &mut facts);
    facts.symbols.sort_by(|left, right| {
        (left.start_line, &left.kind, &left.name).cmp(&(right.start_line, &right.kind, &right.name))
    });
    facts.edges.sort_by(|left, right| {
        left.payload_json
            .to_string()
            .cmp(&right.payload_json.to_string())
    });
    facts
}

impl LineIndex {
    fn new(text: &str) -> Self {
        let mut starts = vec![0];
        for (index, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                starts.push(index + 1);
            }
        }
        Self { starts }
    }

    fn line_for_byte(&self, byte: usize) -> u32 {
        match self.starts.binary_search(&byte) {
            Ok(index) => (index + 1) as u32,
            Err(index) => index as u32,
        }
        .max(1)
    }

    fn byte_for_line(&self, line: u32) -> usize {
        self.starts
            .get(line.saturating_sub(1) as usize)
            .copied()
            .unwrap_or_else(|| *self.starts.last().unwrap_or(&0))
    }
}

fn visit_tree(
    language: LanguageKind,
    node: Node<'_>,
    source: &[u8],
    line_index: &LineIndex,
    facts: &mut ExtractionFacts,
) {
    if let Some(kind) = symbol_kind(language, node.kind()) {
        if let Some(name) = node_name(node, source) {
            let start_line = (node.start_position().row + 1) as u32;
            let end_line = (node.end_position().row + 1) as u32;
            facts.symbols.push(SymbolFact {
                kind: kind.to_string(),
                name: name.clone(),
                qualname: name,
                start_byte: node.start_byte() as u64,
                end_byte: node.end_byte() as u64,
                start_line,
                end_line,
                visibility: node_visibility(node, source),
                signature: node_signature(node, source),
                doc_excerpt: None,
            });
        }
    }

    if let Some(edge_kind) = import_edge_kind(language, node.kind()) {
        let import_text = normalize_ws(node_text(node, source));
        facts.edges.push(EdgeFact {
            edge_kind: edge_kind.to_string(),
            payload_json: serde_json::json!({
                "line": line_index.line_for_byte(node.start_byte()),
                "text": clip(&import_text, 500)
            }),
        });
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit_tree(language, child, source, line_index, facts);
    }
}

fn symbol_kind(language: LanguageKind, node_kind: &str) -> Option<&'static str> {
    match (language, node_kind) {
        (LanguageKind::Rust, "function_item") => Some("function"),
        (LanguageKind::Rust, "struct_item") => Some("struct"),
        (LanguageKind::Rust, "enum_item") => Some("enum"),
        (LanguageKind::Rust, "trait_item") => Some("trait"),
        (LanguageKind::Rust, "const_item") => Some("constant"),
        (LanguageKind::Rust, "static_item") => Some("static"),
        (LanguageKind::Rust, "mod_item") => Some("module"),
        (
            LanguageKind::TypeScript
            | LanguageKind::Tsx
            | LanguageKind::JavaScript
            | LanguageKind::Jsx,
            "function_declaration",
        ) => Some("function"),
        (
            LanguageKind::TypeScript
            | LanguageKind::Tsx
            | LanguageKind::JavaScript
            | LanguageKind::Jsx,
            "method_definition",
        ) => Some("method"),
        (
            LanguageKind::TypeScript
            | LanguageKind::Tsx
            | LanguageKind::JavaScript
            | LanguageKind::Jsx,
            "class_declaration",
        ) => Some("class"),
        (LanguageKind::TypeScript | LanguageKind::Tsx, "interface_declaration") => {
            Some("interface")
        }
        (LanguageKind::TypeScript | LanguageKind::Tsx, "type_alias_declaration") => {
            Some("type_alias")
        }
        (LanguageKind::TypeScript | LanguageKind::Tsx, "enum_declaration") => Some("enum"),
        (
            LanguageKind::TypeScript
            | LanguageKind::Tsx
            | LanguageKind::JavaScript
            | LanguageKind::Jsx,
            "variable_declarator",
        ) => Some("variable"),
        (LanguageKind::Python, "function_definition") => Some("function"),
        (LanguageKind::Python, "class_definition") => Some("class"),
        _ => None,
    }
}

fn import_edge_kind(language: LanguageKind, node_kind: &str) -> Option<&'static str> {
    match (language, node_kind) {
        (LanguageKind::Rust, "use_declaration") => Some("import"),
        (LanguageKind::Rust, "extern_crate_declaration") => Some("import"),
        (
            LanguageKind::TypeScript
            | LanguageKind::Tsx
            | LanguageKind::JavaScript
            | LanguageKind::Jsx,
            "import_statement",
        ) => Some("import"),
        (LanguageKind::Python, "import_statement") => Some("import"),
        (LanguageKind::Python, "import_from_statement") => Some("import"),
        _ => None,
    }
}

fn node_name(node: Node<'_>, source: &[u8]) -> Option<String> {
    node.child_by_field_name("name")
        .map(|child| normalize_ws(node_text(child, source)))
        .filter(|name| !name.is_empty())
}

fn node_visibility(node: Node<'_>, source: &[u8]) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "visibility_modifier" {
            let visibility = normalize_ws(node_text(child, source));
            if !visibility.is_empty() {
                return Some(visibility);
            }
        }
    }
    None
}

fn node_signature(node: Node<'_>, source: &[u8]) -> Option<String> {
    let text = node_text(node, source);
    let first_line = text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default();
    if first_line.is_empty() {
        None
    } else {
        Some(clip(&normalize_ws(first_line), 240))
    }
}

fn node_text<'a>(node: Node<'_>, source: &'a [u8]) -> &'a str {
    std::str::from_utf8(&source[node.start_byte()..node.end_byte()]).unwrap_or_default()
}

fn extract_markdown_headings(text: &str, line_index: &LineIndex, facts: &mut ExtractionFacts) {
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('#') {
            continue;
        }
        let level = trimmed.chars().take_while(|value| *value == '#').count();
        if level == 0
            || level > 6
            || !trimmed
                .chars()
                .nth(level)
                .is_some_and(|value| value.is_whitespace())
        {
            continue;
        }
        let line_number = (index + 1) as u32;
        let name = trimmed[level..].trim().to_string();
        if name.is_empty() {
            continue;
        }
        let start_byte = line_index.byte_for_line(line_number);
        facts.symbols.push(SymbolFact {
            kind: format!("heading{level}"),
            name: name.clone(),
            qualname: name,
            start_byte: start_byte as u64,
            end_byte: (start_byte + line.len()) as u64,
            start_line: line_number,
            end_line: line_number,
            visibility: None,
            signature: None,
            doc_excerpt: None,
        });
    }
}

fn extract_shell_symbols_and_edges(
    text: &str,
    line_index: &LineIndex,
    facts: &mut ExtractionFacts,
) {
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        let line_number = (index + 1) as u32;
        let start_byte = line_index.byte_for_line(line_number);

        if let Some(name) = shell_function_name(trimmed) {
            facts.symbols.push(SymbolFact {
                kind: "function".to_string(),
                name: name.clone(),
                qualname: name,
                start_byte: start_byte as u64,
                end_byte: (start_byte + line.len()) as u64,
                start_line: line_number,
                end_line: line_number,
                visibility: None,
                signature: Some(clip(&normalize_ws(trimmed), 240)),
                doc_excerpt: None,
            });
        }

        if trimmed.starts_with("source ") || trimmed.starts_with(". ") {
            facts.edges.push(EdgeFact {
                edge_kind: "include".to_string(),
                payload_json: serde_json::json!({
                    "line": line_number,
                    "text": clip(&normalize_ws(trimmed), 500)
                }),
            });
        }
    }
}

fn shell_function_name(trimmed: &str) -> Option<String> {
    if let Some(rest) = trimmed.strip_prefix("function ") {
        return rest
            .split(|value: char| value.is_whitespace() || value == '(' || value == '{')
            .find(|value| !value.is_empty())
            .map(ToString::to_string);
    }

    let name = trimmed.split_once("()")?.0.trim();
    if name.is_empty()
        || name
            .chars()
            .any(|value| !(value.is_ascii_alphanumeric() || value == '_' || value == '-'))
    {
        return None;
    }
    Some(name.to_string())
}

fn extract_markers(text: &str, line_index: &LineIndex, facts: &mut ExtractionFacts) {
    for (index, line) in text.lines().enumerate() {
        let line_number = (index + 1) as u32;
        for marker in ["TODO", "FIXME"] {
            if line.contains(marker) {
                let start_byte = line_index.byte_for_line(line_number);
                facts.symbols.push(SymbolFact {
                    kind: "marker".to_string(),
                    name: marker.to_string(),
                    qualname: format!("{marker}:{line_number}"),
                    start_byte: start_byte as u64,
                    end_byte: (start_byte + line.len()) as u64,
                    start_line: line_number,
                    end_line: line_number,
                    visibility: None,
                    signature: None,
                    doc_excerpt: None,
                });
            }
        }
    }
}

fn compute_metrics(language: LanguageKind, text: &str) -> FileMetrics {
    let mut total_lines = 0;
    let mut blank_lines = 0;
    let mut comment_lines = 0;
    let mut todo_count = 0;
    let mut fixme_count = 0;

    for line in text.lines() {
        total_lines += 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blank_lines += 1;
        }
        if is_comment_line(language, trimmed) {
            comment_lines += 1;
        }
        if line.contains("TODO") {
            todo_count += 1;
        }
        if line.contains("FIXME") {
            fixme_count += 1;
        }
    }

    FileMetrics {
        total_lines,
        blank_lines,
        comment_lines,
        todo_count,
        fixme_count,
    }
}

fn is_comment_line(language: LanguageKind, trimmed: &str) -> bool {
    match language {
        LanguageKind::Rust
        | LanguageKind::TypeScript
        | LanguageKind::Tsx
        | LanguageKind::JavaScript
        | LanguageKind::Jsx
        | LanguageKind::Json => {
            trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*')
        }
        LanguageKind::Python | LanguageKind::Shell | LanguageKind::Yaml => trimmed.starts_with('#'),
        LanguageKind::Toml => trimmed.starts_with('#'),
        LanguageKind::Markdown => trimmed.starts_with("<!--"),
        LanguageKind::UnknownText => false,
    }
}

fn normalize_ws(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn clip(value: &str, max_len: usize) -> String {
    if value.len() <= max_len {
        value.to_string()
    } else {
        let mut clipped = value
            .char_indices()
            .take_while(|(index, _)| *index < max_len)
            .map(|(_, value)| value)
            .collect::<String>();
        clipped.push_str("...");
        clipped
    }
}
