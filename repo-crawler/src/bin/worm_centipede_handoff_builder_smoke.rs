
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
        println!("Worm Centipede handoff builder smoke");

        let gitmodules_fixture = r#"
[submodule "linked-repo"]
    path = linked-repo
    url = ../linked-repo
"#;

        let emission = worm_adapter_extractors::parse_gitmodules(
            "Boswecw/Cortex",
            ".gitmodules",
            gitmodules_fixture,
        );

        let edges = emission
            .get("emittedEdges")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let resolutions = match worm_resolution_pipeline::resolve_emitted_edges(&emission) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("FAIL  {}", err);
                std::process::exit(1);
            }
        };

        let bundle = match worm_bundle_builder::build_bundle(
            "Boswecw/Cortex",
            "bundle-smoke-handoff-01",
            &edges,
            &resolutions,
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

        let count = handoff
            .get("candidateIssueKeys")
            .and_then(|v| v.as_array())
            .map(|v| v.len())
            .unwrap_or(0);

        if count != 1 {
            eprintln!("FAIL  expected 1 candidate issue key, got {}", count);
            std::process::exit(1);
        }

        let issue_key = handoff
            .get("candidateIssueKeys")
            .and_then(|v| v.as_array())
            .and_then(|v| v.first())
            .and_then(|v| v.get("issueKey"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if !issue_key.contains("ambiguous_target_identity") {
            eprintln!(
                "FAIL  expected issueKey to include ambiguous_target_identity, got '{}'",
                issue_key
            );
            std::process::exit(1);
        }

        println!("OK  bundle -> Centipede handoff");
        println!("Validated Worm Centipede handoff builder smoke successfully.");
    }
