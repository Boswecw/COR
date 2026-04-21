use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use repo_crawler::config::RepoCrawlerConfig;
use repo_crawler::discovery::discover_repo;
use repo_crawler::scanner::{run_scan, ScanMode};
use repo_crawler::store::Store;

#[test]
fn scan_is_incremental_and_extracts_symbols() {
    let root = temp_repo("incremental");
    write_file(
        &root.join("src/lib.rs"),
        "use std::fs;\n\npub struct Registry;\n\npub fn registry() {}\n// TODO: revisit\n",
    );

    let first = run_scan(&root, None, ScanMode::default()).expect("first scan");
    assert_eq!(first.files_changed, 1);
    assert_eq!(first.files_parsed, 1);

    let store = open_store(&root);
    let repo = discover_repo(&root).expect("repo");
    let repo_record = store
        .repo_by_root(&repo.root_path)
        .expect("repo query")
        .expect("repo indexed");
    let symbols = store
        .query_symbols(repo_record.repo_id, "Registry")
        .expect("symbol query");
    assert!(symbols
        .iter()
        .any(|symbol| symbol.name == "Registry" && symbol.kind == "struct"));

    let second = run_scan(&root, None, ScanMode::default()).expect("second scan");
    assert_eq!(second.files_changed, 0);
    assert_eq!(second.files_parsed, 0);

    write_file(
        &root.join("src/lib.rs"),
        "use std::path::Path;\n\npub struct Registry;\npub fn registry_v2() {}\n",
    );
    let changed = run_scan(
        &root,
        None,
        ScanMode {
            changed_only: true,
            paths: Vec::new(),
            staged_only: false,
        },
    )
    .expect("changed scan");
    assert_eq!(changed.files_changed, 1);
    assert_eq!(changed.files_parsed, 1);

    fs::remove_file(root.join("src/lib.rs")).expect("remove source");
    let deleted = run_scan(&root, None, ScanMode::default()).expect("delete scan");
    assert_eq!(deleted.deleted_files, vec!["src/lib.rs"]);
}

#[test]
fn gitignore_and_policy_skips_are_reported() {
    let root = temp_repo("ignore");
    write_file(&root.join(".gitignore"), "ignored.rs\n");
    write_file(&root.join("ignored.rs"), "pub fn ignored() {}\n");
    write_file(&root.join("kept.rs"), "pub fn kept() {}\n");
    write_file(&root.join("image.png"), "\0not text");

    let report = run_scan(&root, None, ScanMode::default()).expect("scan");
    assert!(report
        .files_skipped
        .iter()
        .any(|skip| skip.rel_path == "image.png" && skip.reason == "excluded_extension:png"));

    let store = open_store(&root);
    let repo = discover_repo(&root).expect("repo");
    let repo_record = store
        .repo_by_root(&repo.root_path)
        .expect("repo query")
        .expect("repo indexed");
    let files = store
        .query_files(repo_record.repo_id, Some("rust"), None)
        .expect("file query");
    assert_eq!(
        files
            .iter()
            .map(|file| file.rel_path.as_str())
            .collect::<Vec<_>>(),
        vec!["kept.rs"]
    );
}

#[test]
fn malformed_supported_file_records_parse_error_diagnostics() {
    let root = temp_repo("parse-error");
    write_file(&root.join("bad.rs"), "fn {\n");

    let report = run_scan(&root, None, ScanMode::default()).expect("scan");
    assert_eq!(report.files_changed, 1);
    assert_eq!(report.files_parsed, 0);

    let store = open_store(&root);
    let repo = discover_repo(&root).expect("repo");
    let repo_record = store
        .repo_by_root(&repo.root_path)
        .expect("repo query")
        .expect("repo indexed");
    let files = store
        .query_files(repo_record.repo_id, Some("rust"), Some("parse_error"))
        .expect("file query");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].rel_path, "bad.rs");

    let export = store
        .export_scan(report.scan_id, true)
        .expect("scan export");
    assert!(!export.diagnostics.is_empty());
    assert!(export
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "syntax_error" || diagnostic.code == "missing_node"));
}

fn open_store(root: &Path) -> Store {
    let repo = discover_repo(root).expect("repo");
    let config = RepoCrawlerConfig::load(&repo.root_path, None).expect("config");
    Store::open(&config.store_path(&repo.root_path)).expect("store")
}

fn temp_repo(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "repo-crawler-test-{name}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(root.join(".git")).expect("temp git dir");
    root
}

fn write_file(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent dirs");
    }
    fs::write(path, body).expect("write file");
}
