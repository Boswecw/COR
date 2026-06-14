use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

use repo_crawler::{
    worm_adapter_extractors,
    worm_bundle_builder,
    worm_centipede_handoff_builder,
    worm_resolution_pipeline,
};
use serde_json::{json, Value};

const MAX_NESTED_REQUIREMENTS_FILES: usize = 32;
const MAX_NESTED_REQUIREMENTS_DEPTH: usize = 8;

#[derive(Default)]
struct SurfaceSummary {
    files_processed: Vec<String>,
    adapter_edge_counts: BTreeMap<String, usize>,
    source_artifact_edge_counts: BTreeMap<String, usize>,
    total_edges_before_resolution: usize,
    total_resolutions: usize,
}

fn maybe_read(path: &Path) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }

    fs::read_to_string(path)
        .map(Some)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }

    normalized
}

fn relative_label(repo_root: &Path, path: &Path) -> String {
    match path.strip_prefix(repo_root) {
        Ok(value) => value.to_string_lossy().replace('\\', "/"),
        Err(_) => path.to_string_lossy().to_string(),
    }
}

fn canonical_file_within_repo(repo_root: &Path, path: &Path) -> Result<PathBuf, String> {
    let canonical_root = fs::canonicalize(repo_root)
        .map_err(|e| format!("failed to canonicalize repo root {}: {}", repo_root.display(), e))?;

    let canonical_file = fs::canonicalize(path)
        .map_err(|e| format!("failed to canonicalize {}: {}", path.display(), e))?;

    if !canonical_file.starts_with(&canonical_root) {
        return Err(format!(
            "symlink-aware containment blocked path outside repo root: {} -> {}",
            path.display(),
            canonical_file.display()
        ));
    }

    Ok(canonical_file)
}

fn resolve_nested_requirement_target(
    repo_root: &Path,
    current_file: &Path,
    include_path: &str,
) -> Result<PathBuf, String> {
    let include_value = include_path.trim();
    if include_value.is_empty() {
        return Err(format!(
            "empty nested requirements include in {}",
            current_file.display()
        ));
    }

    let include_as_path = Path::new(include_value);
    if include_as_path.is_absolute() {
        return Err(format!(
            "nested requirements include must be relative and inside repo root: {} from {}",
            include_value,
            current_file.display()
        ));
    }

    let base_dir = current_file.parent().unwrap_or(repo_root);
    let candidate = normalize_path(&base_dir.join(include_as_path));
    let normalized_root = normalize_path(repo_root);

    if !candidate.starts_with(&normalized_root) {
        return Err(format!(
            "nested requirements include escapes repo root: {} from {}",
            include_value,
            current_file.display()
        ));
    }

    Ok(candidate)
}

fn collect_nested_requirement_files(repo_root: &Path, seeds: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let normalized_root = normalize_path(repo_root);
    let mut visited = BTreeSet::new();
    let mut queued = VecDeque::new();
    let mut ordered = Vec::new();

    for seed in seeds {
        queued.push_back((normalize_path(seed), 0usize));
    }

    while let Some((path, depth)) = queued.pop_front() {
        let canonicalish = path.to_string_lossy().to_string();
        if !visited.insert(canonicalish) {
            continue;
        }

        if visited.len() > MAX_NESTED_REQUIREMENTS_FILES {
            return Err(format!(
                "nested requirements follow exceeded file limit of {} under {}",
                MAX_NESTED_REQUIREMENTS_FILES,
                normalized_root.display()
            ));
        }

        if !path.starts_with(&normalized_root) {
            return Err(format!(
                "nested requirements candidate escaped repo root: {}",
                path.display()
            ));
        }

        if !path.exists() {
            continue;
        }

        let canonical_file = canonical_file_within_repo(&normalized_root, &path)?;
        ordered.push(path.clone());

        let text = fs::read_to_string(&canonical_file)
            .map_err(|e| format!("failed to read nested requirements {}: {}", canonical_file.display(), e))?;

        for raw_line in text.lines() {
            let line = raw_line.trim();
            let include = if let Some(rest) = line.strip_prefix("-r ") {
                Some(rest.trim())
            } else if let Some(rest) = line.strip_prefix("--requirement ") {
                Some(rest.trim())
            } else {
                None
            };

            let Some(include_path) = include else {
                continue;
            };

            if depth + 1 > MAX_NESTED_REQUIREMENTS_DEPTH {
                return Err(format!(
                    "nested requirements depth exceeded {} at {}",
                    MAX_NESTED_REQUIREMENTS_DEPTH,
                    path.display()
                ));
            }

            let next = resolve_nested_requirement_target(&normalized_root, &path, include_path)?;
            queued.push_back((next, depth + 1));
        }
    }

    ordered.sort();
    Ok(ordered)
}

fn accumulate_summary(summary: &mut SurfaceSummary, emission: &Value) {
    let adapter_name = emission
        .get("adapterName")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown_adapter")
        .to_string();

    let source_path = emission
        .get("sourceArtifact")
        .and_then(|v| v.get("path"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown_source")
        .to_string();

    let edge_count = emission
        .get("emittedEdges")
        .and_then(|v| v.as_array())
        .map(|v| v.len())
        .unwrap_or(0);

    summary.files_processed.push(source_path.clone());
    *summary.adapter_edge_counts.entry(adapter_name).or_insert(0) += edge_count;
    *summary.source_artifact_edge_counts.entry(source_path).or_insert(0) += edge_count;
    summary.total_edges_before_resolution += edge_count;
}

fn push_emission(
    all_edges: &mut Vec<Value>,
    all_resolutions: &mut Vec<Value>,
    summary: &mut SurfaceSummary,
    emission: &Value,
) -> Result<(), String> {
    accumulate_summary(summary, emission);

    if let Some(edges) = emission.get("emittedEdges").and_then(|v| v.as_array()) {
        for edge in edges {
            all_edges.push(edge.clone());
        }
    }

    let resolutions = worm_resolution_pipeline::resolve_emitted_edges(emission)?;
    summary.total_resolutions += resolutions.len();

    for resolution in resolutions {
        all_resolutions.push(resolution);
    }

    Ok(())
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|e| format!("failed to serialize {}: {}", path.display(), e))?;

    fs::write(path, text).map_err(|e| format!("failed to write {}: {}", path.display(), e))
}

fn process_file(
    source_repo: &str,
    repo_root: &Path,
    label: &str,
    path: &Path,
    all_edges: &mut Vec<Value>,
    all_resolutions: &mut Vec<Value>,
    summary: &mut SurfaceSummary,
) -> Result<(), String> {
    if !path.exists() {
        println!("SKIP  {}", path.display());
        return Ok(());
    }

    let read_path = canonical_file_within_repo(repo_root, path)?;
    let Some(text) = maybe_read(&read_path)? else {
        println!("SKIP  {}", path.display());
        return Ok(());
    };

    let emission_result = match label {
        ".gitmodules" => Ok(worm_adapter_extractors::parse_gitmodules(source_repo, label, &text)),
        "package.json" => worm_adapter_extractors::parse_package_manifest(source_repo, label, &text),
        "Cargo.toml" => worm_adapter_extractors::parse_cargo_manifest(source_repo, label, &text),
        "pyproject.toml" => worm_adapter_extractors::parse_pyproject_manifest(source_repo, label, &text),
        _ if label.ends_with(".txt") && label.contains("requirements") => {
            worm_adapter_extractors::parse_requirements_manifest(source_repo, label, &text)
        }
        _ if label.starts_with(".github/workflows/") => {
            worm_adapter_extractors::parse_github_workflow(source_repo, label, &text)
        }
        _ => return Err(format!("unsupported repo surface label: {}", label)),
    };

    let emission = emission_result?;
    push_emission(all_edges, all_resolutions, summary, &emission)?;
    println!("OK  read {}", path.display());
    Ok(())
}

fn build_summary_json(source_repo: &str, summary: &SurfaceSummary) -> Value {
    json!({
        "kind": "worm_repo_surface_summary",
        "schemaVersion": 1,
        "sourceRepo": source_repo,
        "filesProcessed": summary.files_processed,
        "adapterEdgeCounts": summary.adapter_edge_counts,
        "sourceArtifactEdgeCounts": summary.source_artifact_edge_counts,
        "totals": {
            "edgesBeforeResolution": summary.total_edges_before_resolution,
            "resolutions": summary.total_resolutions
        },
        "posture": "evidence_bound",
        "timestamp": "2026-04-21T19:55:00-04:00"
    })
}

fn classify_failure_kind(message: &str) -> &str {
    if message.contains("symlink-aware containment blocked path outside repo root") {
        "symlink_escape"
    } else if message.contains("nested requirements include escapes repo root") {
        "nested_requirements_repo_root_escape"
    } else if message.contains("nested requirements depth exceeded") {
        "nested_requirements_depth_limit"
    } else {
        "repo_surface_failure"
    }
}

fn candidate_issue_key(failure_kind: &str) -> String {
    format!("worm.repo_surface.{}", failure_kind)
}

fn failure_severity(failure_kind: &str) -> &'static str {
    match failure_kind {
        "symlink_escape" | "nested_requirements_repo_root_escape" => "high",
        "nested_requirements_depth_limit" => "medium",
        _ => "medium",
    }
}

fn failure_recommended_route(failure_kind: &str) -> &'static str {
    match failure_kind {
        "symlink_escape" | "nested_requirements_repo_root_escape" => "containment",
        "nested_requirements_depth_limit" => "operator_review",
        _ => "operator_review",
    }
}

fn write_failure_outputs(out_dir: &Path, source_repo: &str, repo_root: &Path, message: &str) {
    if let Err(err) = fs::create_dir_all(out_dir) {
        eprintln!(
            "FAIL  could not create failure artifact directory {}: {}",
            out_dir.display(),
            err
        );
        return;
    }

    let failure_kind = classify_failure_kind(message);
    let failure = json!({
        "kind": "worm_repo_surface_failure",
        "schemaVersion": 1,
        "sourceRepo": source_repo,
        "repoRoot": repo_root.display().to_string(),
        "failureKind": failure_kind,
        "message": message,
        "posture": "fail_closed",
        "timestamp": "2026-04-21T20:10:00-04:00"
    });

    let handoff = json!({
        "kind": "worm_centipede_failure_handoff",
        "schemaVersion": 1,
        "sourceRepo": source_repo,
        "repoRoot": repo_root.display().to_string(),
        "failureKind": failure_kind,
        "candidateIssueKeys": [candidate_issue_key(failure_kind)],
        "severity": failure_severity(failure_kind),
        "blocking": true,
        "recommendedRoute": failure_recommended_route(failure_kind),
        "message": message,
        "posture": "fail_closed",
        "timestamp": "2026-04-21T20:10:00-04:00"
    });

    let failure_path = out_dir.join("surface_failure.json");
    match write_json(&failure_path, &failure) {
        Ok(()) => println!("OK  wrote {}", failure_path.display()),
        Err(err) => eprintln!("FAIL  could not write {}: {}", failure_path.display(), err),
    }

    let handoff_path = out_dir.join("centipede_failure_handoff.json");
    match write_json(&handoff_path, &handoff) {
        Ok(()) => println!("OK  wrote {}", handoff_path.display()),
        Err(err) => eprintln!("FAIL  could not write {}: {}", handoff_path.display(), err),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        eprintln!("usage: cargo run --bin worm_run_repo_surface -- <source_repo> <repo_root> <out_dir>");
        std::process::exit(1);
    }

    let source_repo = &args[1];
    let repo_root = PathBuf::from(&args[2]);
    let out_dir = PathBuf::from(&args[3]);
    let normalized_repo_root = match fs::canonicalize(&repo_root) {
        Ok(value) => normalize_path(&value),
        Err(_) => normalize_path(&repo_root),
    };

    println!("Worm run repo surface");
    println!("source repo: {}", source_repo);
    println!("repo root: {}", normalized_repo_root.display());
    println!("output dir: {}", out_dir.display());

    let mut all_edges = Vec::new();
    let mut all_resolutions = Vec::new();
    let mut summary = SurfaceSummary::default();

    let fixed_surfaces = [
        (normalized_repo_root.join(".gitmodules"), ".gitmodules".to_string()),
        (normalized_repo_root.join("package.json"), "package.json".to_string()),
        (normalized_repo_root.join("Cargo.toml"), "Cargo.toml".to_string()),
        (normalized_repo_root.join("pyproject.toml"), "pyproject.toml".to_string()),
    ];

    for (path, label) in fixed_surfaces {
        if let Err(err) = process_file(source_repo, &normalized_repo_root, &label, &path, &mut all_edges, &mut all_resolutions, &mut summary) {
            write_failure_outputs(&out_dir, source_repo, &normalized_repo_root, &err);
            eprintln!("FAIL  {}", err);
            std::process::exit(1);
        }
    }

    let requirement_seeds = vec![
        normalized_repo_root.join("requirements.txt"),
        normalized_repo_root.join("requirements-dev.txt"),
    ];
    let requirement_paths = match collect_nested_requirement_files(&normalized_repo_root, &requirement_seeds) {
        Ok(value) => value,
        Err(err) => {
            write_failure_outputs(&out_dir, source_repo, &normalized_repo_root, &err);
            eprintln!("FAIL  {}", err);
            std::process::exit(1);
        }
    };

    for path in requirement_paths {
        let label = relative_label(&normalized_repo_root, &path);
        if let Err(err) = process_file(source_repo, &normalized_repo_root, &label, &path, &mut all_edges, &mut all_resolutions, &mut summary) {
            write_failure_outputs(&out_dir, source_repo, &normalized_repo_root, &err);
            eprintln!("FAIL  {}", err);
            std::process::exit(1);
        }
    }

    let workflows_dir = normalized_repo_root.join(".github/workflows");
    if workflows_dir.is_dir() {
        let entries = match fs::read_dir(&workflows_dir) {
            Ok(value) => value,
            Err(err) => {
                let message = format!("could not read {}: {}", workflows_dir.display(), err);
                write_failure_outputs(&out_dir, source_repo, &normalized_repo_root, &message);
                eprintln!("FAIL  {}", message);
                std::process::exit(1);
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(value) => value,
                Err(err) => {
                    let message = format!("could not enumerate workflow entry: {}", err);
                    write_failure_outputs(&out_dir, source_repo, &normalized_repo_root, &message);
                    eprintln!("FAIL  {}", message);
                    std::process::exit(1);
                }
            };

            let path = entry.path();
            let ext = path.extension().and_then(|v| v.to_str()).unwrap_or_default();
            if ext != "yml" && ext != "yaml" {
                continue;
            }

            let relative = relative_label(&normalized_repo_root, &path);
            if let Err(err) = process_file(source_repo, &normalized_repo_root, &relative, &path, &mut all_edges, &mut all_resolutions, &mut summary) {
                write_failure_outputs(&out_dir, source_repo, &normalized_repo_root, &err);
                eprintln!("FAIL  {}", err);
                std::process::exit(1);
            }
        }
    } else {
        println!("SKIP  {}", workflows_dir.display());
    }

    let bundle = match worm_bundle_builder::build_bundle(
        source_repo,
        "bundle-run-repo-surface-11",
        &all_edges,
        &all_resolutions,
    ) {
        Ok(value) => value,
        Err(err) => {
            write_failure_outputs(&out_dir, source_repo, &normalized_repo_root, &err);
            eprintln!("FAIL  {}", err);
            std::process::exit(1);
        }
    };

    let handoff = match worm_centipede_handoff_builder::build_handoff(&bundle) {
        Ok(value) => value,
        Err(err) => {
            write_failure_outputs(&out_dir, source_repo, &normalized_repo_root, &err);
            eprintln!("FAIL  {}", err);
            std::process::exit(1);
        }
    };

    let summary_json = build_summary_json(source_repo, &summary);

    if let Err(err) = fs::create_dir_all(&out_dir) {
        eprintln!("FAIL  could not create {}: {}", out_dir.display(), err);
        std::process::exit(1);
    }

    let bundle_path = out_dir.join("bundle.json");
    let handoff_path = out_dir.join("handoff.json");
    let summary_path = out_dir.join("surface_summary.json");

    if let Err(err) = write_json(&bundle_path, &bundle) {
        write_failure_outputs(&out_dir, source_repo, &normalized_repo_root, &err);
        eprintln!("FAIL  {}", err);
        std::process::exit(1);
    }
    if let Err(err) = write_json(&handoff_path, &handoff) {
        write_failure_outputs(&out_dir, source_repo, &normalized_repo_root, &err);
        eprintln!("FAIL  {}", err);
        std::process::exit(1);
    }
    if let Err(err) = write_json(&summary_path, &summary_json) {
        write_failure_outputs(&out_dir, source_repo, &normalized_repo_root, &err);
        eprintln!("FAIL  {}", err);
        std::process::exit(1);
    }

    println!("OK  wrote {}", bundle_path.display());
    println!("OK  wrote {}", handoff_path.display());
    println!("OK  wrote {}", summary_path.display());
    println!("Validated Worm repo surface run successfully.");
}
