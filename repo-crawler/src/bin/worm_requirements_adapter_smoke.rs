
    use repo_crawler::worm_adapter_extractors;

    fn main() {
        println!("Worm requirements adapter smoke");

        let fixture = r#"
# core
fastapi==0.115.0
git+https://github.com/Boswecw/sharedkit.git
my-lib @ git+ssh://git@github.com/Boswecw/my-lib.git
-r requirements-dev.txt
"#;

        let emission = match worm_adapter_extractors::parse_requirements_manifest(
            "Boswecw/DataForge",
            "requirements.txt",
            fixture,
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
            eprintln!("FAIL  expected 2 requirements manifest edges, got {}", edges);
            std::process::exit(1);
        }

        println!("OK  requirements_manifest_parse");
        println!("Validated Worm requirements adapter smoke successfully.");
    }
