
use std::path::PathBuf;

#[path = "../worm_contracts.rs"]
mod worm_contracts;

fn main() {
    let repo_root = PathBuf::from(".");
    let mut validated_files = 0usize;

    println!("Worm contract smoke check");
    println!("repo root: {}", repo_root.display());

    for set in worm_contracts::WORM_CONTRACT_SETS {
        match worm_contracts::validate_contract_set(&repo_root, set) {
            Ok(count) => {
                validated_files += count;
                println!("OK  {} ({count} files)", set.label);
            }
            Err(err) => {
                eprintln!("FAIL  {} :: {}", set.label, err);
                std::process::exit(1);
            }
        }
    }

    println!("Validated Worm contract example sets successfully.");
    println!("Total files checked: {}", validated_files);
}
