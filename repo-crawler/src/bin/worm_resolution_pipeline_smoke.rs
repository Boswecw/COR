
    #[path = "../worm_adapter_extractors.rs"]
    mod worm_adapter_extractors;
    #[path = "../worm_target_normalizer.rs"]
    mod worm_target_normalizer;
    #[path = "../worm_resolution_pipeline.rs"]
    mod worm_resolution_pipeline;

    fn main() {
        println!("Worm resolution pipeline smoke");

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

        let git_resolutions = match worm_resolution_pipeline::resolve_emitted_edges(&git_emission) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("FAIL  {}", err);
                std::process::exit(1);
            }
        };

        let pkg_resolutions = match worm_resolution_pipeline::resolve_emitted_edges(&pkg_emission) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("FAIL  {}", err);
                std::process::exit(1);
            }
        };

        if git_resolutions.len() != 1 {
            eprintln!("FAIL  expected 1 git resolution, got {}", git_resolutions.len());
            std::process::exit(1);
        }

        if pkg_resolutions.len() != 1 {
            eprintln!("FAIL  expected 1 package resolution, got {}", pkg_resolutions.len());
            std::process::exit(1);
        }

        let pkg_display = pkg_resolutions[0]
            .get("canonicalIdentity")
            .and_then(|v| v.get("display"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if pkg_display != "Boswecw/shared-lib" {
            eprintln!(
                "FAIL  expected package resolution display Boswecw/shared-lib, got '{}'",
                pkg_display
            );
            std::process::exit(1);
        }

        println!("OK  gitmodules extraction -> resolution");
        println!("OK  package manifest extraction -> resolution");
        println!("Validated Worm resolution pipeline smoke successfully.");
    }
