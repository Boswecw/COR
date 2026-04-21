
    #[path = "../worm_adapter_extractors.rs"]
    mod worm_adapter_extractors;

    fn main() {
        println!("Worm pyproject adapter smoke");

        let pyproject_fixture = r#"
[project]
dependencies = [
  "fastapi>=0.110.0",
  "my-lib @ git+https://github.com/Boswecw/my-lib.git"
]

[project.optional-dependencies]
dev = [
  "another-lib @ git+ssh://git@github.com/Boswecw/another-lib.git"
]

[tool.poetry.dependencies]
python = "^3.12"
sharedkit = { git = "https://github.com/Boswecw/sharedkit.git" }

[tool.poetry.group.dev.dependencies]
tooling = { git = "https://github.com/Boswecw/tooling.git" }
"#;

        let emission = match worm_adapter_extractors::parse_pyproject_manifest(
            "Boswecw/DataForge",
            "pyproject.toml",
            pyproject_fixture,
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

        if edges != 4 {
            eprintln!("FAIL  expected 4 pyproject manifest edges, got {}", edges);
            std::process::exit(1);
        }

        println!("OK  pyproject_manifest_parse");
        println!("Validated Worm pyproject adapter smoke successfully.");
    }
