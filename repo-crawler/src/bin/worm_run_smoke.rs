
    use std::env;
    use std::fs;
    use std::path::PathBuf;

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

    fn main() {
        let out_dir = env::args()
            .nth(1)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp/worm-run-smoke-out"));

        println!("Worm run smoke");
        println!("output dir: {}", out_dir.display());

        let gitmodules_fixture = r#"
[submodule "linked-repo"]
    path = linked-repo
    url = ../linked-repo
"#;

        let package_fixture = r#"
{
  "dependencies": {
    "shared-lib": "git+ssh://git@github.com/Boswecw/shared-lib.git",
    "forge-contract-core": "Boswecw/forge-contract-core",
    "lodash": "^4.17.21"
  }
}
"#;

        let git_emission = worm_adapter_extractors::parse_gitmodules(
            "Boswecw/Cortex",
            ".gitmodules",
            gitmodules_fixture,
        );

        let package_emission = match worm_adapter_extractors::parse_package_manifest(
            "Boswecw/Cortex",
            "package.json",
            package_fixture,
        ) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("FAIL  {}", err);
                std::process::exit(1);
            }
        };

        let mut all_edges = Vec::new();
        let mut all_resolutions = Vec::new();

        for emission in [&git_emission, &package_emission] {
            if let Some(edges) = emission.get("emittedEdges").and_then(|v| v.as_array()) {
                for edge in edges {
                    all_edges.push(edge.clone());
                }
            }

            let resolutions = match worm_resolution_pipeline::resolve_emitted_edges(emission) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("FAIL  {}", err);
                    std::process::exit(1);
                }
            };

            for resolution in resolutions {
                all_resolutions.push(resolution);
            }
        }

        let bundle = match worm_bundle_builder::build_bundle(
            "Boswecw/Cortex",
            "bundle-run-smoke-01",
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
            eprintln!("FAIL  could not create output directory {}: {}", out_dir.display(), err);
            std::process::exit(1);
        }

        let bundle_path = out_dir.join("bundle.json");
        let handoff_path = out_dir.join("handoff.json");

        let bundle_text = match serde_json::to_string_pretty(&bundle) {
            Ok(text) => text,
            Err(err) => {
                eprintln!("FAIL  could not serialize bundle: {}", err);
                std::process::exit(1);
            }
        };

        let handoff_text = match serde_json::to_string_pretty(&handoff) {
            Ok(text) => text,
            Err(err) => {
                eprintln!("FAIL  could not serialize handoff: {}", err);
                std::process::exit(1);
            }
        };

        if let Err(err) = fs::write(&bundle_path, bundle_text) {
            eprintln!("FAIL  could not write {}: {}", bundle_path.display(), err);
            std::process::exit(1);
        }

        if let Err(err) = fs::write(&handoff_path, handoff_text) {
            eprintln!("FAIL  could not write {}: {}", handoff_path.display(), err);
            std::process::exit(1);
        }

        println!("OK  wrote {}", bundle_path.display());
        println!("OK  wrote {}", handoff_path.display());
        println!("Validated Worm run smoke successfully.");
    }
