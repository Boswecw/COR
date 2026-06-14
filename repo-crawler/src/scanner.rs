use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;

use ignore::WalkBuilder;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::config::RepoCrawlerConfig;
use crate::discovery::{discover_repo, staged_paths, RepoIdentity};
use crate::error::Result;
use crate::extract;
use crate::lang::{classify_path, LanguageKind};
use crate::parser::{ParseDiagnostic, ParserRegistry};
use crate::policy::{Policy, SkippedPath};
use crate::store::{FileUpsert, Store, SCHEMA_VERSION};

#[derive(Debug, Clone, Default)]
pub struct ScanMode {
    pub changed_only: bool,
    pub paths: Vec<PathBuf>,
    pub staged_only: bool,
}

#[derive(Debug, Serialize)]
pub struct ScanReport {
    pub schema_version: String,
    pub scan_id: i64,
    pub repo: RepoIdentity,
    pub mode: String,
    pub files_seen: u64,
    pub files_changed: u64,
    pub files_parsed: u64,
    pub files_skipped: Vec<SkippedPath>,
    pub deleted_files: Vec<String>,
    pub errors: Vec<String>,
    pub store_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChangeState {
    Added,
    Changed,
    Unchanged,
}

pub fn run_scan(
    root_arg: &Path,
    config_path: Option<&Path>,
    mut mode: ScanMode,
) -> Result<ScanReport> {
    let repo = discover_repo(root_arg)?;
    let config = RepoCrawlerConfig::load(&repo.root_path, config_path)?;
    if mode.staged_only {
        mode.paths = staged_paths(&repo.root_path)?;
        mode.changed_only = true;
    }

    let store_path = config.store_path(&repo.root_path);
    let store = Store::open(&store_path)?;
    let repo_id = store.ensure_repo(&repo)?;
    let scan_id = store.begin_scan_run(repo_id)?;
    if mode.staged_only && mode.paths.is_empty() {
        store.finish_scan_run(scan_id, "ready", 0, 0, 0, &[])?;
        return Ok(ScanReport {
            schema_version: SCHEMA_VERSION.to_string(),
            scan_id,
            repo,
            mode: mode_name(&mode),
            files_seen: 0,
            files_changed: 0,
            files_parsed: 0,
            files_skipped: Vec::new(),
            deleted_files: Vec::new(),
            errors: Vec::new(),
            store_path,
        });
    }
    let policy = Arc::new(Policy::new(&config));
    let normalized_targets = normalize_targets(&repo.root_path, &mode.paths);

    let skipped = Arc::new(Mutex::new(Vec::<SkippedPath>::new()));
    let mut files_seen = 0;
    let mut files_changed = 0;
    let mut files_parsed = 0;
    let mut errors = Vec::new();
    let mut current_seen = BTreeSet::new();

    let mut builder = WalkBuilder::new(&repo.root_path);
    builder
        .hidden(!config.crawl.include_hidden)
        .git_ignore(config.crawl.respect_gitignore)
        .git_global(config.crawl.respect_gitignore)
        .git_exclude(config.crawl.respect_gitignore)
        .parents(config.crawl.respect_gitignore)
        .follow_links(config.repo.follow_symlinks);

    let root_for_filter = repo.root_path.clone();
    let skipped_for_filter = Arc::clone(&skipped);
    let policy_for_filter = Arc::clone(&policy);
    builder.filter_entry(move |entry| {
        if entry.depth() == 0 {
            return true;
        }
        let is_dir = entry
            .file_type()
            .is_some_and(|file_type| file_type.is_dir());
        if !is_dir {
            return true;
        }
        let rel_path = entry
            .path()
            .strip_prefix(&root_for_filter)
            .unwrap_or_else(|_| entry.path());
        if let Some(reason) = policy_for_filter.skip_dir_reason(rel_path) {
            push_skip(&skipped_for_filter, rel_path, reason);
            false
        } else {
            true
        }
    });

    for entry_result in builder.build() {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(error) => {
                errors.push(error.to_string());
                continue;
            }
        };

        if entry.depth() == 0 {
            continue;
        }
        let rel_path = entry
            .path()
            .strip_prefix(&repo.root_path)
            .unwrap_or_else(|_| entry.path())
            .to_path_buf();
        if !target_matches(&rel_path, &normalized_targets) {
            continue;
        }

        let Some(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_file() {
            continue;
        }

        match process_file(
            &repo,
            repo_id,
            scan_id,
            &config,
            &policy,
            &store,
            &mode,
            &rel_path,
            entry.path(),
        ) {
            Ok(FileProcessOutcome::Skipped(reason)) => {
                push_skip(&skipped, &rel_path, reason);
            }
            Ok(FileProcessOutcome::Indexed {
                rel_path,
                change_state,
                parsed,
            }) => {
                files_seen += 1;
                current_seen.insert(rel_path);
                if change_state != ChangeState::Unchanged {
                    files_changed += 1;
                }
                if parsed {
                    files_parsed += 1;
                }
            }
            Err(error) => {
                errors.push(format!("{}: {error}", rel_string(&rel_path)));
            }
        }
    }

    let mut deleted_files = Vec::new();
    for previous in store.active_files(repo_id)? {
        let rel_path = PathBuf::from(&previous.rel_path);
        if !target_matches(&rel_path, &normalized_targets) {
            continue;
        }
        if !current_seen.contains(&previous.rel_path) {
            store.mark_deleted(previous.file_id, scan_id)?;
            deleted_files.push(previous.rel_path.clone());
            files_changed += 1;
        }
    }

    let status = if errors.is_empty() {
        "ready"
    } else {
        "partial_success"
    };
    store.finish_scan_run(
        scan_id,
        status,
        files_seen,
        files_changed,
        files_parsed,
        &errors,
    )?;

    let mut files_skipped = match Arc::try_unwrap(skipped) {
        Ok(mutex) => mutex.into_inner().unwrap_or_default(),
        Err(arc) => arc.lock().map(|guard| guard.clone()).unwrap_or_default(),
    };
    files_skipped.sort_by(|left, right| left.rel_path.cmp(&right.rel_path));
    deleted_files.sort();

    Ok(ScanReport {
        schema_version: SCHEMA_VERSION.to_string(),
        scan_id,
        repo,
        mode: mode_name(&mode),
        files_seen,
        files_changed,
        files_parsed,
        files_skipped,
        deleted_files,
        errors,
        store_path,
    })
}

enum FileProcessOutcome {
    Skipped(String),
    Indexed {
        rel_path: String,
        change_state: ChangeState,
        parsed: bool,
    },
}

fn process_file(
    repo: &RepoIdentity,
    repo_id: i64,
    scan_id: i64,
    config: &RepoCrawlerConfig,
    policy: &Policy,
    store: &Store,
    mode: &ScanMode,
    rel_path: &Path,
    abs_path: &Path,
) -> Result<FileProcessOutcome> {
    let metadata = fs::metadata(abs_path)?;
    let size_bytes = metadata.len();
    if let Some(reason) = policy.skip_file_reason(rel_path, size_bytes, false) {
        return Ok(FileProcessOutcome::Skipped(reason));
    }

    let content = fs::read(abs_path)?;
    let is_binary = content.iter().take(8192).any(|byte| *byte == 0);
    if let Some(reason) = policy.skip_file_reason(rel_path, size_bytes, is_binary) {
        return Ok(FileProcessOutcome::Skipped(reason));
    }

    let rel_path_string = rel_string(rel_path);
    let sha256 = sha256_hex(&content);
    let language = classify_path(rel_path);
    let previous = store.get_file(repo_id, &rel_path_string)?;
    let change_state = match previous
        .as_ref()
        .and_then(|record| record.sha256.as_deref())
    {
        None => ChangeState::Added,
        Some(previous_hash) if previous_hash == sha256 => ChangeState::Unchanged,
        Some(_) => ChangeState::Changed,
    };
    let changed_in_scan = change_state != ChangeState::Unchanged
        || previous
            .as_ref()
            .is_some_and(|record| record.parse_status == "deleted");

    if !changed_in_scan {
        let previous = previous.expect("unchanged files have a previous record");
        store.upsert_file(&FileUpsert {
            repo_id,
            rel_path: rel_path_string.clone(),
            abs_path: abs_path.to_string_lossy().to_string(),
            size_bytes,
            mtime_ns: mtime_ns(&metadata),
            sha256,
            lang: previous.lang,
            parser_id: previous.parser_id,
            is_binary: previous.is_binary,
            parse_status: previous.parse_status,
            scan_id,
            changed_in_scan: false,
        })?;
        return Ok(FileProcessOutcome::Indexed {
            rel_path: rel_path_string,
            change_state,
            parsed: false,
        });
    }

    if mode.changed_only && change_state == ChangeState::Unchanged {
        return Ok(FileProcessOutcome::Indexed {
            rel_path: rel_path_string,
            change_state,
            parsed: false,
        });
    }

    let mut parser_id = None;
    let mut parse_status = "unsupported".to_string();
    let mut diagnostics = Vec::<ParseDiagnostic>::new();
    let mut parsed = false;
    let mut tree_for_extraction = None;

    if language == LanguageKind::UnknownText {
        diagnostics.push(ParseDiagnostic {
            severity: "info".to_string(),
            code: "unsupported_language".to_string(),
            message: "language classifier returned unknown text".to_string(),
            start_line: None,
            end_line: None,
            payload_json: serde_json::json!({ "language": language.id() }),
        });
    } else if !config.language_enabled(language.id()) {
        diagnostics.push(ParseDiagnostic {
            severity: "info".to_string(),
            code: "language_disabled".to_string(),
            message: format!("{} is disabled in config", language.id()),
            start_line: None,
            end_line: None,
            payload_json: serde_json::json!({ "language": language.id() }),
        });
    } else {
        let parse = ParserRegistry::parse(language, &content)?;
        parser_id = parse.parser_id.clone();
        diagnostics.extend(parse.diagnostics.clone());
        if parse.unsupported_reason.is_some() {
            parse_status = "unsupported".to_string();
        } else if parse.parse_success {
            parse_status = "parsed".to_string();
            parsed = true;
            tree_for_extraction = parse.tree;
        } else {
            parse_status = "parse_error".to_string();
        }
    }

    let file_id = store.upsert_file(&FileUpsert {
        repo_id,
        rel_path: rel_path_string.clone(),
        abs_path: repo.root_path.join(rel_path).to_string_lossy().to_string(),
        size_bytes,
        mtime_ns: mtime_ns(&metadata),
        sha256,
        lang: language.id().to_string(),
        parser_id: parser_id.clone(),
        is_binary,
        parse_status,
        scan_id,
        changed_in_scan: true,
    })?;
    store.replace_file_facts(file_id)?;
    store.insert_diagnostics(scan_id, file_id, &diagnostics)?;

    let extraction = extract::extract(language, &content, tree_for_extraction.as_ref());
    store.insert_symbols(scan_id, file_id, &extraction.symbols)?;
    store.insert_edges(scan_id, repo_id, file_id, &extraction.edges)?;
    store.upsert_metrics(scan_id, file_id, &extraction.metrics)?;

    Ok(FileProcessOutcome::Indexed {
        rel_path: rel_path_string,
        change_state,
        parsed,
    })
}

fn push_skip(skipped: &Arc<Mutex<Vec<SkippedPath>>>, rel_path: &Path, reason: String) {
    if let Ok(mut guard) = skipped.lock() {
        guard.push(SkippedPath {
            rel_path: rel_string(rel_path),
            reason,
        });
    }
}

fn normalize_targets(root: &Path, paths: &[PathBuf]) -> Vec<PathBuf> {
    paths
        .iter()
        .map(|path| {
            if path.is_absolute() {
                path.strip_prefix(root).unwrap_or(path).to_path_buf()
            } else {
                path.to_path_buf()
            }
        })
        .map(normalize_relative_path)
        .collect()
}

fn normalize_relative_path(path: PathBuf) -> PathBuf {
    let mut output = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(value) => output.push(value),
            _ => {}
        }
    }
    output
}

fn target_matches(rel_path: &Path, targets: &[PathBuf]) -> bool {
    targets.is_empty()
        || targets
            .iter()
            .any(|target| rel_path == target || rel_path.starts_with(target))
}

fn rel_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn mtime_ns(metadata: &fs::Metadata) -> i64 {
    metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos().min(i64::MAX as u128) as i64)
        .unwrap_or_default()
}

fn mode_name(mode: &ScanMode) -> String {
    if mode.staged_only {
        "staged-only".to_string()
    } else if !mode.paths.is_empty() {
        "path-targeted".to_string()
    } else if mode.changed_only {
        "changed-only".to_string()
    } else {
        "full".to_string()
    }
}
