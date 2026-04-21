
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[path = "../worm_adapter_extractors.rs"]
mod worm_adapter_extractors;
#[path = "../worm_target_normalizer.rs"]
mod worm_target_normalizer;
#[path = "../worm_resolution_pipeline.rs"]
mod worm_resolution_pipeline;
#[path = "../worm_bundle_builder.rs"]
mod worm_bundle_builder;
#[path = "../worm_centipede_handoff_builder.rs"]
mod worm_centipede_handoff_builder;

fn read_optional(path: &str) -> Result<Option<String>, String> {
    if path == "-" {
        return Ok(None);
    }

    fs::read_to_string(path)
        .map(Some)
        .map_err(|e| format!("failed to read {}: {}", path, e))
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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        eprintln!(
            "usage: cargo run --bin worm_run_from_files -- <source_repo> <gitmodules_path_or_dash> <package_json_path_or_dash> <out_dir>"
        );
        std::process::exit(1);
    }

    let source_repo = &args[1];
    let gitmodules_path = &args[2];
    let package_json_path = &args[3];
    let out_dir = PathBuf::from(&args[4]);

    println!("Worm run from files");
    println!("source repo: {}", source_repo);
    println!("gitmodules: {}", gitmodules_path);
    println!("package.json: {}", package_json_path);
    println!("output dir: {}", out_dir.display());

    let gitmodules_text = match read_optional(gitmodules_path) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("FAIL  {}", err);
            std::process::exit(1);
        }
    };

    let package_json_text = match read_optional(package_json_path) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("FAIL  {}", err);
            std::process::exit(1);
        }
    };

    let mut all_edges = Vec::new();
    let mut all_resolutions = Vec::new();

    if let Some(text) = gitmodules_text {
        let emission = worm_adapter_extractors::parse_gitmodules(
            source_repo,
            ".gitmodules",
            &text,
        );

        if let Err(err) = push_emission(&mut all_edges, &mut all_resolutions, &emission) {
            eprintln!("FAIL  {}", err);
            std::process::exit(1);
        }
    }

    if let Some(text) = package_json_text {
        let emission = match worm_adapter_extractors::parse_package_manifest(
            source_repo,
            "package.json",
            &text,
        ) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("FAIL  {}", err);
                std::process::exit(1);
            }
        };

        if let Err(err) = push_emission(&mut all_edges, &mut all_resolutions, &emission) {
            eprintln!("FAIL  {}", err);
            std::process::exit(1);
        }
    }

    let bundle = match worm_bundle_builder::build_bundle(
        source_repo,
        "bundle-run-from-files-01",
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
    println!("Validated Worm run from files successfully.");
}
