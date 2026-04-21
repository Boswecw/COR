
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use repo_crawler::{
    worm_adapter_extractors,
    worm_bundle_builder,
    worm_centipede_handoff_builder,
    worm_resolution_pipeline,
};

fn maybe_read(path: &Path) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }

    fs::read_to_string(path)
        .map(Some)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))
}

fn push_emission(
    all_edges: &mut Vec<serde_json::Value>,
    all_resolutions: &mut Vec<serde_json::Value>,
    emission: &serde_json::Value,
) -> Result<(), String> {
    if let Some(edges) = emission.get("emittedEdges").and_then(|v| v.as_array()) {
        for edge in edges {
            all_edges.push(edge.clone());
        }
    }

    let resolutions = worm_resolution_pipeline::resolve_emitted_edges(emission)?;
    for resolution in resolutions {
        all_resolutions.push(resolution);
    }

    Ok(())
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|e| format!("failed to serialize {}: {}", path.display(), e))?;

    fs::write(path, text)
        .map_err(|e| format!("failed to write {}: {}", path.display(), e))
}

fn process_file(
    source_repo: &str,
    label: &str,
    path: &Path,
    all_edges: &mut Vec<serde_json::Value>,
    all_resolutions: &mut Vec<serde_json::Value>,
) -> Result<(), String> {
    let Some(text) = maybe_read(path)? else {
        println!("SKIP  {}", path.display());
        return Ok(());
    };

    let emission_result = match label {
        ".gitmodules" => Ok(worm_adapter_extractors::parse_gitmodules(
            source_repo,
            label,
            &text,
        )),
        "package.json" => worm_adapter_extractors::parse_package_manifest(
            source_repo,
            label,
            &text,
        ),
        "Cargo.toml" => worm_adapter_extractors::parse_cargo_manifest(
            source_repo,
            label,
            &text,
        ),
        "pyproject.toml" => worm_adapter_extractors::parse_pyproject_manifest(
            source_repo,
            label,
            &text,
        ),
        "requirements.txt" | "requirements-dev.txt" => {
            worm_adapter_extractors::parse_requirements_manifest(
                source_repo,
                label,
                &text,
            )
        }
        _ if label.starts_with(".github/workflows/") => {
            worm_adapter_extractors::parse_github_workflow(
                source_repo,
                label,
                &text,
            )
        }
        _ => return Err(format!("unsupported repo surface label: {}", label)),
    };

    let emission = emission_result?;
    push_emission(all_edges, all_resolutions, &emission)?;
    println!("OK  read {}", path.display());
    Ok(())
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

    println!("Worm run repo surface");
    println!("source repo: {}", source_repo);
    println!("repo root: {}", repo_root.display());
    println!("output dir: {}", out_dir.display());

    let mut all_edges = Vec::new();
    let mut all_resolutions = Vec::new();

    let surfaces = [
        (repo_root.join(".gitmodules"), ".gitmodules"),
        (repo_root.join("package.json"), "package.json"),
        (repo_root.join("Cargo.toml"), "Cargo.toml"),
        (repo_root.join("pyproject.toml"), "pyproject.toml"),
        (repo_root.join("requirements.txt"), "requirements.txt"),
        (repo_root.join("requirements-dev.txt"), "requirements-dev.txt"),
    ];

    for (path, label) in surfaces {
        if let Err(err) = process_file(
            source_repo,
            label,
            &path,
            &mut all_edges,
            &mut all_resolutions,
        ) {
            eprintln!("FAIL  {}", err);
            std::process::exit(1);
        }
    }

    let workflows_dir = repo_root.join(".github/workflows");
    if workflows_dir.is_dir() {
        let entries = match fs::read_dir(&workflows_dir) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("FAIL  could not read {}: {}", workflows_dir.display(), err);
                std::process::exit(1);
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(value) => value,
                Err(err) => {
                    eprintln!("FAIL  could not enumerate workflow entry: {}", err);
                    std::process::exit(1);
                }
            };

            let path = entry.path();
            let ext = path.extension().and_then(|v| v.to_str()).unwrap_or_default();
            if ext != "yml" && ext != "yaml" {
                continue;
            }

            let relative = match path.strip_prefix(&repo_root) {
                Ok(value) => value.to_string_lossy().replace('\\', "/"),
                Err(_) => path.to_string_lossy().to_string(),
            };

            if let Err(err) = process_file(
                source_repo,
                &relative,
                &path,
                &mut all_edges,
                &mut all_resolutions,
            ) {
                eprintln!("FAIL  {}", err);
                std::process::exit(1);
            }
        }
    } else {
        println!("SKIP  {}", workflows_dir.display());
    }

    let bundle = match worm_bundle_builder::build_bundle(
        source_repo,
        "bundle-run-repo-surface-05",
        &all_edges,
        &all_resolutions,
    ) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("FAIL  {}", err);
            std::process::exit(1);
        }
    };

    let handoff = match worm_centipede_handoff_builder::build_handoff(&bundle) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("FAIL  {}", err);
            std::process::exit(1);
        }
    };

    if let Err(err) = fs::create_dir_all(&out_dir) {
        eprintln!("FAIL  could not create {}: {}", out_dir.display(), err);
        std::process::exit(1);
    }

    let bundle_path = out_dir.join("bundle.json");
    let handoff_path = out_dir.join("handoff.json");

    if let Err(err) = write_json(&bundle_path, &bundle) {
        eprintln!("FAIL  {}", err);
        std::process::exit(1);
    }

    if let Err(err) = write_json(&handoff_path, &handoff) {
        eprintln!("FAIL  {}", err);
        std::process::exit(1);
    }

    println!("OK  wrote {}", bundle_path.display());
    println!("OK  wrote {}", handoff_path.display());
    println!("Validated Worm repo surface run successfully.");
}
