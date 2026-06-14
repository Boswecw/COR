use std::fs;
use std::process::Command;

fn run_case(root: &std::path::Path, out: &std::path::Path) -> bool {
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

fn main() {
    println!("Worm nested requirements guardrails smoke");

    let escape_root = std::env::temp_dir().join("worm-nested-guardrails-escape");
    let escape_out = std::env::temp_dir().join("worm-nested-guardrails-escape-out");
    let _ = fs::remove_dir_all(&escape_root);
    let _ = fs::remove_dir_all(&escape_out);
    fs::create_dir_all(&escape_root).expect("create escape root");
    fs::write(escape_root.join("requirements.txt"), "-r ../outside.txt\n").expect("write escape requirements");
    fs::write(std::env::temp_dir().join("outside.txt"), "git+https://github.com/Boswecw/outside.git\n")
        .expect("write outside file");

    if run_case(&escape_root, &escape_out) {
        eprintln!("FAIL  expected repo-root escape case to fail");
        std::process::exit(1);
    }

    let depth_root = std::env::temp_dir().join("worm-nested-guardrails-depth");
    let depth_out = std::env::temp_dir().join("worm-nested-guardrails-depth-out");
    let _ = fs::remove_dir_all(&depth_root);
    let _ = fs::remove_dir_all(&depth_out);
    fs::create_dir_all(depth_root.join("requirements")).expect("create depth requirements dir");

    fs::write(depth_root.join("requirements.txt"), "-r requirements/1.txt\n").expect("write seed");
    for i in 1..=9 {
        let next_line = if i == 9 {
            "git+https://github.com/Boswecw/depth-final.git\n".to_string()
        } else {
            format!("-r {}.txt\n", i + 1)
        };
        fs::write(depth_root.join("requirements").join(format!("{}.txt", i)), next_line)
            .expect("write nested depth file");
    }

    if run_case(&depth_root, &depth_out) {
        eprintln!("FAIL  expected nested depth case to fail");
        std::process::exit(1);
    }

    println!("OK  boundary_escape_blocked");
    println!("OK  depth_limit_blocked");
    println!("Validated Worm nested requirements guardrails smoke successfully.");
}
