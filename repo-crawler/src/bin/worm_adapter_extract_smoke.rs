
    #[path = "../worm_adapter_extractors.rs"]
    mod worm_adapter_extractors;

    fn main() {
        println!("Worm adapter extract smoke");

        let gitmodules_fixture = r#"
[submodule "linked-repo"]
    path = linked-repo
    url = git@github.com:Boswecw/linked-repo.git
"#;

        let package_fixture = r#"
{
  "dependencies": {
    "shared-lib": "git+ssh://git@github.com/Boswecw/shared-lib.git",
    "lodash": "^4.17.21"
  }
}
"#;

        let git_emission = worm_adapter_extractors::parse_gitmodules(
            "Boswecw/Cortex",
            ".gitmodules",
            gitmodules_fixture,
        );

        let git_edges = git_emission
            .get("emittedEdges")
            .and_then(|v| v.as_array())
            .map(|v| v.len())
            .unwrap_or(0);

        if git_edges != 1 {
            eprintln!("FAIL  expected 1 gitmodules edge, got {}", git_edges);
            std::process::exit(1);
        }

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

        let pkg_edges = pkg_emission
            .get("emittedEdges")
            .and_then(|v| v.as_array())
            .map(|v| v.len())
            .unwrap_or(0);

        let pkg_skipped = pkg_emission
            .get("skippedReferences")
            .and_then(|v| v.as_array())
            .map(|v| v.len())
            .unwrap_or(0);

        if pkg_edges != 1 {
            eprintln!("FAIL  expected 1 package manifest edge, got {}", pkg_edges);
            std::process::exit(1);
        }

        if pkg_skipped != 1 {
            eprintln!("FAIL  expected 1 skipped package manifest reference, got {}", pkg_skipped);
            std::process::exit(1);
        }

        println!("OK  gitmodules_parse");
        println!("OK  package_manifest_parse");
        println!("Validated Worm adapter extraction smoke successfully.");
    }
