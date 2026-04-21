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

fn read_handoff(out: &Path) -> Value {
    let text = fs::read_to_string(out.join("centipede_failure_handoff.json"))
        .expect("read centipede_failure_handoff.json");
    serde_json::from_str(&text).expect("parse handoff json")
}

fn main() {
    println!("Worm Centipede failure handoff smoke");

    let lexical_root = std::env::temp_dir().join("worm-centipede-failure-lexical-root");
    let lexical_out = std::env::temp_dir().join("worm-centipede-failure-lexical-out");
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

    let lexical = read_handoff(&lexical_out);
    assert_eq!(lexical["kind"], "worm_centipede_failure_handoff");
    assert_eq!(lexical["failureKind"], "nested_requirements_repo_root_escape");
    assert_eq!(lexical["severity"], "high");
    assert_eq!(lexical["blocking"], true);
    assert_eq!(lexical["recommendedRoute"], "containment");
    assert_eq!(lexical["candidateIssueKeys"][0], "worm.repo_surface.nested_requirements_repo_root_escape");

    let symlink_root = std::env::temp_dir().join("worm-centipede-failure-symlink-root");
    let symlink_out = std::env::temp_dir().join("worm-centipede-failure-symlink-out");
    let symlink_outside = std::env::temp_dir().join("worm-centipede-failure-symlink-outside");
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

    let symlink = read_handoff(&symlink_out);
    assert_eq!(symlink["failureKind"], "symlink_escape");
    assert_eq!(symlink["severity"], "high");
    assert_eq!(symlink["blocking"], true);
    assert_eq!(symlink["recommendedRoute"], "containment");
    assert_eq!(symlink["candidateIssueKeys"][0], "worm.repo_surface.symlink_escape");

    println!("OK  lexical_failure_handoff_written");
    println!("OK  symlink_failure_handoff_written");
    println!("Validated Worm Centipede failure handoff smoke successfully.");
}
