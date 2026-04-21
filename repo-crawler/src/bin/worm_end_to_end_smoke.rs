
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
        println!("Worm end-to-end smoke");

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
            "bundle-e2e-01",
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

        let edge_count = bundle.get("edges").and_then(|v| v.as_array()).map(|v| v.len()).unwrap_or(0);
        let resolution_count = bundle.get("resolutions").and_then(|v| v.as_array()).map(|v| v.len()).unwrap_or(0);
        let finding_count = bundle.get("findings").and_then(|v| v.as_array()).map(|v| v.len()).unwrap_or(0);
        let candidate_count = handoff.get("candidateIssueKeys").and_then(|v| v.as_array()).map(|v| v.len()).unwrap_or(0);

        if edge_count != 3 {
            eprintln!("FAIL  expected 3 edges, got {}", edge_count);
            std::process::exit(1);
        }

        if resolution_count != 3 {
            eprintln!("FAIL  expected 3 resolutions, got {}", resolution_count);
            std::process::exit(1);
        }

        if finding_count != 1 {
            eprintln!("FAIL  expected 1 finding, got {}", finding_count);
            std::process::exit(1);
        }

        if candidate_count != 1 {
            eprintln!("FAIL  expected 1 candidate issue key, got {}", candidate_count);
            std::process::exit(1);
        }

        println!("OK  edges: {}", edge_count);
        println!("OK  resolutions: {}", resolution_count);
        println!("OK  findings: {}", finding_count);
        println!("OK  candidate issue keys: {}", candidate_count);
        println!("Validated Worm end-to-end smoke successfully.");
    }
