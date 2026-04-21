
    use repo_crawler::worm_adapter_extractors;

    fn main() {
        println!("Worm pyproject uv sources smoke");

        let fixture = r#"
[project]
name = "uv-test"
version = "0.1.0"
dependencies = ["fastapi>=0.110.0"]

[tool.uv.sources]
sharedlib = { git = "https://github.com/Boswecw/sharedlib.git" }
mytool = { git = "ssh://git@github.com/Boswecw/mytool.git" }
"#;

        let emission = match worm_adapter_extractors::parse_pyproject_manifest(
            "Boswecw/DataForge",
            "pyproject.toml",
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
            eprintln!("FAIL  expected 2 uv source edges, got {}", edges);
            std::process::exit(1);
        }

        println!("OK  pyproject_uv_sources_parse");
        println!("Validated Worm pyproject uv sources smoke successfully.");
    }
