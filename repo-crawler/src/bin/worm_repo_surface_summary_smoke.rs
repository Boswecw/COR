
use std::fs;
use std::process::Command;

fn main() {
    println!("Worm repo surface summary smoke");

    let root = std::env::temp_dir().join("worm-repo-surface-summary-smoke");
    let out = std::env::temp_dir().join("worm-repo-surface-summary-smoke-out");

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&out);

    fs::create_dir_all(root.join(".github/workflows")).expect("create root");
    fs::write(
        root.join(".gitmodules"),
        "[submodule \"linked-repo\"]\n    path = linked-repo\n    url = ../linked-repo\n",
    )
    .expect("write gitmodules");
    fs::write(
        root.join("package.json"),
        "{\n  \"dependencies\": {\n    \"shared-lib\": \"Boswecw/shared-lib\"\n  }\n}\n",
    )
    .expect("write package");
    fs::write(
        root.join("pyproject.toml"),
        "[project]\nname = \"x\"\nversion = \"0.1.0\"\ndependencies = [\"sharedpkg @ git+https://github.com/Boswecw/sharedpkg.git\"]\n\n[tool.uv.sources]\nsharedlib = { git = \"https://github.com/Boswecw/sharedlib.git\" }\n",
    )
    .expect("write pyproject");
    fs::write(
        root.join(".github/workflows/ci.yml"),
        "jobs:\n  build:\n    steps:\n      - uses: actions/checkout@v4\n",
    )
    .expect("write workflow");

    let status = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "worm_run_repo_surface",
            "--",
            "Boswecw/Cortex",
            root.to_str().unwrap(),
            out.to_str().unwrap(),
        ])
        .status()
        .expect("spawn worm_run_repo_surface");

    if !status.success() {
        eprintln!("FAIL  worm_run_repo_surface did not succeed");
        std::process::exit(1);
    }

    let summary_path = out.join("surface_summary.json");
    if !summary_path.exists() {
        eprintln!("FAIL  surface_summary.json was not written");
        std::process::exit(1);
    }

    println!("OK  repo_surface_summary_written");
    println!("Validated Worm repo surface summary smoke successfully.");
}
