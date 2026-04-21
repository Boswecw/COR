
    #[path = "../worm_adapter_extractors.rs"]
    mod worm_adapter_extractors;

    fn main() {
        println!("Worm cargo adapter smoke");

        let cargo_fixture = r#"
[package]
name = "demo"
version = "0.1.0"

[dependencies]
forge-contract-core = { git = "https://github.com/Boswecw/forge-contract-core.git" }
serde = "1"

[workspace.dependencies]
dataforge = { git = "https://github.com/Boswecw/DataForge.git" }
"#;

        let emission = match worm_adapter_extractors::parse_cargo_manifest(
            "Boswecw/Cortex",
            "Cargo.toml",
            cargo_fixture,
        ) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("FAIL  {}", err);
                std::process::exit(1);
            }
        };

        let edges = emission
            .get("emittedEdges")
            .and_then(|v| v.as_array())
            .map(|v| v.len())
            .unwrap_or(0);

        if edges != 2 {
            eprintln!("FAIL  expected 2 cargo manifest edges, got {}", edges);
            std::process::exit(1);
        }

        println!("OK  cargo_manifest_parse");
        println!("Validated Worm cargo adapter smoke successfully.");
    }
