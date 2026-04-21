
    use repo_crawler::worm_adapter_extractors;

    fn main() {
        println!("Worm GitHub workflow adapter smoke");

        let fixture = r#"
name: ci
on:
  push:

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: Boswecw/shared-action@main
      - uses: ./local-action
      - uses: docker://alpine:3.20
"#;

        let emission = match worm_adapter_extractors::parse_github_workflow(
            "Boswecw/Cortex",
            ".github/workflows/ci.yml",
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
            eprintln!("FAIL  expected 2 workflow edges, got {}", edges);
            std::process::exit(1);
        }

        println!("OK  github_workflow_parse");
        println!("Validated Worm GitHub workflow adapter smoke successfully.");
    }
