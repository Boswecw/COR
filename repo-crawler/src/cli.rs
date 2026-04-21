use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::config::RepoCrawlerConfig;
use crate::discovery::discover_repo;
use crate::error::{CrawlerError, Result};
use crate::scanner::{run_scan, ScanMode};
use crate::store::Store;
use crate::watch::run_watch;

#[derive(Debug, Parser)]
#[command(name = "repo-crawler")]
#[command(about = "Deterministic mechanical repository crawler and parser")]
pub struct Cli {
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init(InitArgs),
    Scan(ScanArgs),
    Watch(WatchArgs),
    Query(QueryArgs),
    Export(ExportArgs),
    Doctor(DoctorArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(default_value = ".")]
    pub root: PathBuf,
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct ScanArgs {
    pub root: PathBuf,
    #[arg(long)]
    pub changed_only: bool,
    #[arg(long, num_args = 1..)]
    pub paths: Vec<PathBuf>,
    #[arg(long)]
    pub staged_only: bool,
}

#[derive(Debug, Args)]
pub struct WatchArgs {
    pub root: PathBuf,
}

#[derive(Debug, Args)]
pub struct QueryArgs {
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    #[command(subcommand)]
    pub command: QueryCommand,
}

#[derive(Debug, Subcommand)]
pub enum QueryCommand {
    Symbols(QuerySymbolsArgs),
    Files(QueryFilesArgs),
}

#[derive(Debug, Args)]
pub struct QuerySymbolsArgs {
    pub name: String,
}

#[derive(Debug, Args)]
pub struct QueryFilesArgs {
    #[arg(long)]
    pub lang: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    #[command(subcommand)]
    pub command: ExportCommand,
}

#[derive(Debug, Subcommand)]
pub enum ExportCommand {
    Scan(ExportScanArgs),
}

#[derive(Debug, Args)]
pub struct ExportScanArgs {
    #[arg(long)]
    pub scan_id: i64,
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    #[arg(long, value_enum)]
    pub format: Option<ExportFormat>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ExportFormat {
    Json,
    Jsonl,
}

#[derive(Debug, Args)]
pub struct DoctorArgs {
    #[arg(default_value = ".")]
    pub root: PathBuf,
}

pub fn run_cli(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init(args) => {
            let repo = discover_repo(&args.root)?;
            let config_path = RepoCrawlerConfig::write_default(&repo.root_path, args.force)?;
            let config = RepoCrawlerConfig::load(&repo.root_path, Some(&config_path))?;
            let store = Store::open(&config.store_path(&repo.root_path))?;
            let repo_id = store.ensure_repo(&repo)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "ready",
                    "repo_id": repo_id,
                    "config_path": config_path,
                    "store_path": store.path()
                }))?
            );
        }
        Commands::Scan(args) => {
            let report = run_scan(
                &args.root,
                cli.config.as_deref(),
                ScanMode {
                    changed_only: args.changed_only,
                    paths: args.paths,
                    staged_only: args.staged_only,
                },
            )?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Commands::Watch(args) => {
            run_watch(&args.root, cli.config.as_deref())?;
        }
        Commands::Query(args) => {
            let repo = discover_repo(&args.root)?;
            let config = RepoCrawlerConfig::load(&repo.root_path, cli.config.as_deref())?;
            let store = Store::open(&config.store_path(&repo.root_path))?;
            let repo_record = store.repo_by_root(&repo.root_path)?.ok_or_else(|| {
                CrawlerError::Config(format!(
                    "{} has not been scanned yet",
                    repo.root_path.display()
                ))
            })?;
            match args.command {
                QueryCommand::Symbols(symbol_args) => {
                    let rows = store.query_symbols(repo_record.repo_id, &symbol_args.name)?;
                    println!("{}", serde_json::to_string_pretty(&rows)?);
                }
                QueryCommand::Files(file_args) => {
                    let rows = store.query_files(
                        repo_record.repo_id,
                        file_args.lang.as_deref(),
                        file_args.status.as_deref(),
                    )?;
                    println!("{}", serde_json::to_string_pretty(&rows)?);
                }
            }
        }
        Commands::Export(args) => match args.command {
            ExportCommand::Scan(scan_args) => {
                let repo = discover_repo(&scan_args.root)?;
                let config = RepoCrawlerConfig::load(&repo.root_path, cli.config.as_deref())?;
                let format = scan_args.format.unwrap_or_else(|| {
                    match config.export.default_format.as_str() {
                        "jsonl" => ExportFormat::Jsonl,
                        _ => ExportFormat::Json,
                    }
                });
                let store = Store::open(&config.store_path(&repo.root_path))?;
                let package =
                    store.export_scan(scan_args.scan_id, config.export.include_diagnostics)?;
                match format {
                    ExportFormat::Json => println!("{}", serde_json::to_string_pretty(&package)?),
                    ExportFormat::Jsonl => {
                        println!(
                            "{}",
                            serde_json::to_string(&serde_json::json!({
                                "type": "scan_run",
                                "record": package.scan_run
                            }))?
                        );
                        for file in package.files {
                            println!(
                                "{}",
                                serde_json::to_string(&serde_json::json!({
                                    "type": "file",
                                    "record": file
                                }))?
                            );
                        }
                        for symbol in package.symbols {
                            println!(
                                "{}",
                                serde_json::to_string(&serde_json::json!({
                                    "type": "symbol",
                                    "record": symbol
                                }))?
                            );
                        }
                        for edge in package.edges {
                            println!(
                                "{}",
                                serde_json::to_string(&serde_json::json!({
                                    "type": "edge",
                                    "record": edge
                                }))?
                            );
                        }
                        for diagnostic in package.diagnostics {
                            println!(
                                "{}",
                                serde_json::to_string(&serde_json::json!({
                                    "type": "diagnostic",
                                    "record": diagnostic
                                }))?
                            );
                        }
                        for metrics in package.metrics {
                            println!(
                                "{}",
                                serde_json::to_string(&serde_json::json!({
                                    "type": "metrics",
                                    "record": metrics
                                }))?
                            );
                        }
                    }
                }
            }
        },
        Commands::Doctor(args) => {
            let repo = discover_repo(&args.root)?;
            let config = RepoCrawlerConfig::load(&repo.root_path, cli.config.as_deref())?;
            let store_path = config.store_path(&repo.root_path);
            let store = Store::open(&store_path)?;
            let repo_record = store.repo_by_root(&repo.root_path)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "ready",
                    "root_path": repo.root_path,
                    "vcs_type": repo.vcs_type,
                    "head_ref": repo.head_ref,
                    "head_commit": repo.head_commit,
                    "config_version": config.version,
                    "enabled_languages": config.parse.enabled_languages,
                    "store_path": store.path(),
                    "store_initialized": true,
                    "repo_indexed": repo_record.is_some(),
                    "watch_enabled": config.watch.enabled
                }))?
            );
        }
    }
    Ok(())
}
