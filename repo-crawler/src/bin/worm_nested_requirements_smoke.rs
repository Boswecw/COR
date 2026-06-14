
use std::fs;
use std::process::Command;

fn main() {
    println!("Worm nested requirements smoke");

    let root = std::env::temp_dir().join("worm-nested-requirements-smoke");
    let out = std::env::temp_dir().join("worm-nested-requirements-smoke-out");

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&out);

    fs::create_dir_all(root.join("requirements")).expect("create requirements dir");

    fs::write(
        root.join("requirements.txt"),
        "-r requirements/dev.txt\nfastapi==0.115.0\n",
    )
    .expect("write requirements.txt");

    fs::write(
        root.join("requirements/dev.txt"),
        "git+https://github.com/Boswecw/test-tooling.git\n",
    )
    .expect("write nested dev.txt");

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

    let summary = fs::read_to_string(out.join("surface_summary.json")).expect("read summary");
    if !summary.contains("requirements/dev.txt") {
        eprintln!("FAIL  nested requirements file not attributed in summary");
        std::process::exit(1);
    }

    println!("OK  nested_requirements_followed");
    println!("Validated Worm nested requirements smoke successfully.");
}
