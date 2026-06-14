use std::fs;
use std::os::unix::fs as unix_fs;
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
    println!("Worm symlink containment smoke");

    let file_root = std::env::temp_dir().join("worm-symlink-file-root");
    let file_out = std::env::temp_dir().join("worm-symlink-file-out");
    let file_outside_dir = std::env::temp_dir().join("worm-symlink-file-outside");
    let _ = fs::remove_dir_all(&file_root);
    let _ = fs::remove_dir_all(&file_out);
    let _ = fs::remove_dir_all(&file_outside_dir);
    fs::create_dir_all(&file_root).expect("create file root");
    fs::create_dir_all(&file_outside_dir).expect("create file outside dir");
    fs::write(
        file_outside_dir.join("outside-requirements.txt"),
        "git+https://github.com/Boswecw/outside-file.git\n",
    )
    .expect("write outside requirements file");
    unix_fs::symlink(
        file_outside_dir.join("outside-requirements.txt"),
        file_root.join("requirements.txt"),
    )
    .expect("create requirements file symlink");

    if run_case(&file_root, &file_out) {
        eprintln!("FAIL  expected symlinked requirements file case to fail");
        std::process::exit(1);
    }

    let dir_root = std::env::temp_dir().join("worm-symlink-dir-root");
    let dir_out = std::env::temp_dir().join("worm-symlink-dir-out");
    let dir_outside = std::env::temp_dir().join("worm-symlink-dir-outside");
    let _ = fs::remove_dir_all(&dir_root);
    let _ = fs::remove_dir_all(&dir_out);
    let _ = fs::remove_dir_all(&dir_outside);
    fs::create_dir_all(&dir_root).expect("create dir root");
    fs::create_dir_all(&dir_outside).expect("create dir outside");
    fs::write(dir_root.join("requirements.txt"), "-r requirements/dev.txt\n")
        .expect("write root requirements");
    fs::write(
        dir_outside.join("dev.txt"),
        "git+https://github.com/Boswecw/outside-dir.git\n",
    )
    .expect("write outside nested requirements");
    unix_fs::symlink(&dir_outside, dir_root.join("requirements")).expect("create requirements dir symlink");

    if run_case(&dir_root, &dir_out) {
        eprintln!("FAIL  expected symlinked requirements directory case to fail");
        std::process::exit(1);
    }

    println!("OK  symlinked_requirements_file_blocked");
    println!("OK  symlinked_requirements_directory_blocked");
    println!("Validated Worm symlink containment smoke successfully.");
}
