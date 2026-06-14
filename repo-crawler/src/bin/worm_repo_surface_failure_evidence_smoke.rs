use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

fn run_case(root: &Path, out: &Path) -> bool {
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

    status.success()
}

fn read_failure_kind(out: &Path) -> String {
    let text = fs::read_to_string(out.join("surface_failure.json"))
        .expect("read surface_failure.json");
    let value: Value = serde_json::from_str(&text).expect("parse surface_failure.json");
    assert_eq!(value["kind"], "worm_repo_surface_failure");
    value["failureKind"]
        .as_str()
        .expect("failureKind string")
        .to_string()
}

fn main() {
    println!("Worm repo surface failure evidence smoke");

    let lexical_root = std::env::temp_dir().join("worm-failure-evidence-lexical-root");
    let lexical_out = std::env::temp_dir().join("worm-failure-evidence-lexical-out");
    let _ = fs::remove_dir_all(&lexical_root);
    let _ = fs::remove_dir_all(&lexical_out);
    fs::create_dir_all(&lexical_root).expect("create lexical root");
    fs::write(lexical_root.join("requirements.txt"), "-r ../outside.txt\n")
        .expect("write lexical requirements");
    fs::write(std::env::temp_dir().join("outside.txt"), "git+https://github.com/Boswecw/outside.git\n")
        .expect("write outside file");

    if run_case(&lexical_root, &lexical_out) {
        eprintln!("FAIL  expected lexical escape case to fail");
        std::process::exit(1);
    }

    let lexical_kind = read_failure_kind(&lexical_out);
    if lexical_kind != "nested_requirements_repo_root_escape" {
        eprintln!(
            "FAIL  expected nested_requirements_repo_root_escape, got {}",
            lexical_kind
        );
        std::process::exit(1);
    }

    let symlink_root = std::env::temp_dir().join("worm-failure-evidence-symlink-root");
    let symlink_out = std::env::temp_dir().join("worm-failure-evidence-symlink-out");
    let symlink_outside = std::env::temp_dir().join("worm-failure-evidence-symlink-outside");
    let _ = fs::remove_dir_all(&symlink_root);
    let _ = fs::remove_dir_all(&symlink_out);
    let _ = fs::remove_dir_all(&symlink_outside);
    fs::create_dir_all(&symlink_root).expect("create symlink root");
    fs::create_dir_all(&symlink_outside).expect("create symlink outside");
    fs::write(
        symlink_outside.join("requirements.txt"),
        "git+https://github.com/Boswecw/outside-symlink.git\n",
    )
    .expect("write outside symlink target");
    unix_fs::symlink(
        symlink_outside.join("requirements.txt"),
        symlink_root.join("requirements.txt"),
    )
    .expect("create symlinked requirements file");

    if run_case(&symlink_root, &symlink_out) {
        eprintln!("FAIL  expected symlink escape case to fail");
        std::process::exit(1);
    }

    let symlink_kind = read_failure_kind(&symlink_out);
    if symlink_kind != "symlink_escape" {
        eprintln!("FAIL  expected symlink_escape, got {}", symlink_kind);
        std::process::exit(1);
    }

    println!("OK  lexical_escape_failure_artifact_written");
    println!("OK  symlink_escape_failure_artifact_written");
    println!("Validated Worm repo surface failure evidence smoke successfully.");
}
