
    #[path = "../worm_adapter_extractors.rs"]
    mod worm_adapter_extractors;
    #[path = "../worm_target_normalizer.rs"]
    mod worm_target_normalizer;
    #[path = "../worm_resolution_pipeline.rs"]
    mod worm_resolution_pipeline;
    #[path = "../worm_bundle_builder.rs"]
    mod worm_bundle_builder;

    fn main() {
        println!("Worm bundle builder smoke");

        let gitmodules_fixture = r#"
[submodule "linked-repo"]
    path = linked-repo
    url = ../linked-repo
"#;

        let package_fixture = r#"
{
  "dependencies": {
    "shared-lib": "Boswecw/shared-lib"
  }
}
"#;

        let git_emission = worm_adapter_extractors::parse_gitmodules(
            "Boswecw/Cortex",
            ".gitmodules",
            gitmodules_fixture,
        );

        let pkg_emission = match worm_adapter_extractors::parse_package_manifest(
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

        for emission in [&git_emission, &pkg_emission] {
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
            "bundle-smoke-01",
            &all_edges,
            &all_resolutions,
        ) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("FAIL  {}", err);
                std::process::exit(1);
            }
        };

        let findings = bundle
            .get("findings")
            .and_then(|v| v.as_array())
            .map(|v| v.len())
            .unwrap_or(0);

        if findings != 1 {
            eprintln!("FAIL  expected 1 finding, got {}", findings);
            std::process::exit(1);
        }

        let reason_code = bundle
            .get("findings")
            .and_then(|v| v.as_array())
            .and_then(|v| v.first())
            .and_then(|v| v.get("reasonCode"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if reason_code != "ambiguous_target_identity" {
            eprintln!(
                "FAIL  expected reasonCode ambiguous_target_identity, got '{}'",
                reason_code
            );
            std::process::exit(1);
        }

        println!("OK  extraction -> resolution -> bundle");
        println!("Validated Worm bundle builder smoke successfully.");
    }
