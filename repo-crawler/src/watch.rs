use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

use crate::config::RepoCrawlerConfig;
use crate::discovery::discover_repo;
use crate::error::Result;
use crate::scanner::{run_scan, ScanMode};

pub fn run_watch(root_arg: &Path, config_path: Option<&Path>) -> Result<()> {
    let repo = discover_repo(root_arg)?;
    let config = RepoCrawlerConfig::load(&repo.root_path, config_path)?;
    let debounce = Duration::from_millis(config.watch.debounce_ms);
    let reconcile = Duration::from_secs(config.watch.poll_reconcile_seconds);
    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(&repo.root_path, RecursiveMode::Recursive)?;

    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "status": "watching",
            "root_path": repo.root_path,
            "debounce_ms": config.watch.debounce_ms,
            "poll_reconcile_seconds": config.watch.poll_reconcile_seconds
        }))?
    );

    let mut dirty_paths = BTreeSet::<PathBuf>::new();
    let mut last_event = Instant::now();
    let mut last_reconcile = Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) => {
                for path in event.paths {
                    let rel_path = if path.is_absolute() {
                        path.strip_prefix(&repo.root_path)
                            .unwrap_or(&path)
                            .to_path_buf()
                    } else {
                        path
                    };
                    dirty_paths.insert(rel_path);
                }
                last_event = Instant::now();
            }
            Ok(Err(error)) => {
                eprintln!("repo-crawler watch event error: {error}");
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        if !dirty_paths.is_empty() && last_event.elapsed() >= debounce {
            let paths = dirty_paths.iter().cloned().collect::<Vec<_>>();
            dirty_paths.clear();
            let report = run_scan(
                &repo.root_path,
                config_path,
                ScanMode {
                    changed_only: true,
                    paths,
                    staged_only: false,
                },
            )?;
            println!("{}", serde_json::to_string(&report)?);
            last_reconcile = Instant::now();
        }

        if last_reconcile.elapsed() >= reconcile {
            let report = run_scan(
                &repo.root_path,
                config_path,
                ScanMode {
                    changed_only: true,
                    paths: Vec::new(),
                    staged_only: false,
                },
            )?;
            println!("{}", serde_json::to_string(&report)?);
            last_reconcile = Instant::now();
        }
    }

    Ok(())
}
