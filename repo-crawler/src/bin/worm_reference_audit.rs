
use std::path::PathBuf;

#[path = "../worm_contract_audit.rs"]
mod worm_contract_audit;

fn main() {
    let repo_root = PathBuf::from(".");
    println!("Worm cross-reference audit");
    println!("repo root: {}", repo_root.display());

    match worm_contract_audit::run_reference_audit(&repo_root) {
        Ok(()) => {
            println!("Validated Worm cross-reference audit successfully.");
        }
        Err(err) => {
            eprintln!("FAIL  {}", err);
            std::process::exit(1);
        }
    }
}
