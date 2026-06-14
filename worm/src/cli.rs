use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::adapter::{
    cortex_extraction_observations, read_observations, repo_crawler_observations,
};
use crate::engine::Reconciler;
use crate::error::{Result, WormError};
use crate::governance::observe_governance;
use crate::model::{OperatorReviewState, DEFAULT_STORE_REL_PATH};
use crate::store::{DecisionFilter, ObservationFilter, Store};

#[derive(Debug, Parser)]
#[command(name = "worm")]
#[command(about = "Mechanical weighted multi-surface truth reconciler")]
pub struct Cli {
    #[arg(long, global = true)]
    pub store: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init(InitArgs),
    Ingest(IngestArgs),
    Observe(ObserveArgs),
    Reconcile(ReconcileArgs),
    Query(QueryArgs),
    Export(ExportArgs),
    Review(ReviewArgs),
    Doctor(DoctorArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(default_value = ".")]
    pub root: PathBuf,
}

#[derive(Debug, Args)]
pub struct IngestArgs {
    #[arg(default_value = ".")]
    pub root: PathBuf,
    #[command(subcommand)]
    pub command: IngestCommand,
}

#[derive(Debug, Subcommand)]
pub enum IngestCommand {
    Observations(InputArgs),
    RepoCrawler(InputArgs),
    CortexExtraction(InputArgs),
}

#[derive(Debug, Args)]
pub struct InputArgs {
    #[arg(long)]
    pub input: PathBuf,
}

#[derive(Debug, Args)]
pub struct ObserveArgs {
    #[arg(default_value = ".")]
    pub root: PathBuf,
    #[command(subcommand)]
    pub command: ObserveCommand,
}

#[derive(Debug, Subcommand)]
pub enum ObserveCommand {
    Governance,
}

#[derive(Debug, Args)]
pub struct ReconcileArgs {
    #[arg(default_value = ".")]
    pub root: PathBuf,
    #[arg(long)]
    pub repository_id: Option<String>,
    #[arg(long)]
    pub revision_id: Option<String>,
    #[arg(long)]
    pub claim_class: Option<String>,
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub since_ts: Option<i64>,
}

#[derive(Debug, Args)]
pub struct QueryArgs {
    #[arg(default_value = ".")]
    pub root: PathBuf,
    #[command(subcommand)]
    pub command: QueryCommand,
}

#[derive(Debug, Subcommand)]
pub enum QueryCommand {
    Observations(QueryObservationsArgs),
    Decisions(QueryDecisionsArgs),
}

#[derive(Debug, Args)]
pub struct QueryObservationsArgs {
    #[arg(long)]
    pub repository_id: Option<String>,
    #[arg(long)]
    pub revision_id: Option<String>,
    #[arg(long)]
    pub claim_class: Option<String>,
    #[arg(long)]
    pub target: Option<String>,
    #[arg(long)]
    pub since_ts: Option<i64>,
}

#[derive(Debug, Args)]
pub struct QueryDecisionsArgs {
    #[arg(long)]
    pub run_id: Option<i64>,
    #[arg(long)]
    pub disposition: Option<String>,
    #[arg(long)]
    pub claim_class: Option<String>,
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    #[arg(default_value = ".")]
    pub root: PathBuf,
    #[command(subcommand)]
    pub command: ExportCommand,
}

#[derive(Debug, Subcommand)]
pub enum ExportCommand {
    Run(ExportRunArgs),
}

#[derive(Debug, Args)]
pub struct ExportRunArgs {
    #[arg(long)]
    pub run_id: i64,
    #[arg(long, value_enum, default_value = "json")]
    pub format: ExportFormat,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ExportFormat {
    Json,
    Jsonl,
}

#[derive(Debug, Args)]
pub struct ReviewArgs {
    #[arg(default_value = ".")]
    pub root: PathBuf,
    #[arg(long)]
    pub decision_id: i64,
    #[arg(long, value_enum)]
    pub state: ReviewStateArg,
    #[arg(long)]
    pub reviewer: Option<String>,
    #[arg(long)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ReviewStateArg {
    Accepted,
    Rejected,
    Deferred,
    NeedsFollowUp,
    HistoricalOnly,
}

#[derive(Debug, Args)]
pub struct DoctorArgs {
    #[arg(default_value = ".")]
    pub root: PathBuf,
}

pub fn run_cli(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init(args) => {
            let store = open_store(&args.root, cli.store.as_deref())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "ready",
                    "store_path": store.path(),
                    "contract": crate::model::CONTRACT_VERSION
                }))?
            );
        }
        Commands::Ingest(args) => {
            let store = open_store(&args.root, cli.store.as_deref())?;
            let observations = match args.command {
                IngestCommand::Observations(input) => read_observations(&input.input)?,
                IngestCommand::RepoCrawler(input) => repo_crawler_observations(&input.input)?,
                IngestCommand::CortexExtraction(input) => {
                    cortex_extraction_observations(&input.input)?
                }
            };
            let ids = store.insert_observations(&observations)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "ready",
                    "inserted_observations": ids.len(),
                    "observation_ids": ids
                }))?
            );
        }
        Commands::Observe(args) => {
            let store = open_store(&args.root, cli.store.as_deref())?;
            let observations = match args.command {
                ObserveCommand::Governance => observe_governance(&args.root),
            };
            let ids = store.insert_observations(&observations)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "ready",
                    "inserted_observations": ids.len(),
                    "observation_ids": ids
                }))?
            );
        }
        Commands::Reconcile(args) => {
            let store = open_store(&args.root, cli.store.as_deref())?;
            let observations = store.observations(&ObservationFilter {
                repository_id: args.repository_id,
                revision_id: args.revision_id,
                claim_class: args.claim_class,
                target: args.target,
                since_ts: args.since_ts,
            })?;
            let run_id = store.begin_reconciliation_run(observations.len())?;
            let decisions = Reconciler::default().reconcile(&observations);
            for decision in &decisions {
                store.insert_decision(run_id, decision)?;
            }
            store.finish_reconciliation_run(run_id, "ready", decisions.len())?;
            let report = store.reconciliation_report(run_id)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Commands::Query(args) => {
            let store = open_store(&args.root, cli.store.as_deref())?;
            match args.command {
                QueryCommand::Observations(query) => {
                    let rows = store.observations(&ObservationFilter {
                        repository_id: query.repository_id,
                        revision_id: query.revision_id,
                        claim_class: query.claim_class,
                        target: query.target,
                        since_ts: query.since_ts,
                    })?;
                    println!("{}", serde_json::to_string_pretty(&rows)?);
                }
                QueryCommand::Decisions(query) => {
                    let rows = store.decisions(&DecisionFilter {
                        run_id: query.run_id,
                        disposition: query.disposition,
                        claim_class: query.claim_class,
                    })?;
                    println!("{}", serde_json::to_string_pretty(&rows)?);
                }
            }
        }
        Commands::Export(args) => {
            let store = open_store(&args.root, cli.store.as_deref())?;
            match args.command {
                ExportCommand::Run(run_args) => {
                    let report = store.reconciliation_report(run_args.run_id)?;
                    match run_args.format {
                        ExportFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&report)?)
                        }
                        ExportFormat::Jsonl => {
                            println!(
                                "{}",
                                serde_json::to_string(&serde_json::json!({
                                    "type": "reconciliation_run",
                                    "record": {
                                        "run_id": report.run_id,
                                        "started_at": report.started_at,
                                        "finished_at": report.finished_at,
                                        "status": report.status
                                    }
                                }))?
                            );
                            for decision in report.decisions {
                                println!(
                                    "{}",
                                    serde_json::to_string(&serde_json::json!({
                                        "type": "reconciliation_decision",
                                        "record": decision
                                    }))?
                                );
                            }
                        }
                    }
                }
            }
        }
        Commands::Review(args) => {
            let store = open_store(&args.root, cli.store.as_deref())?;
            let review_state = review_state(args.state);
            store.update_review(
                args.decision_id,
                review_state,
                args.reviewer.as_deref(),
                args.notes.as_deref(),
            )?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "ready",
                    "decision_id": args.decision_id,
                    "review_updated": true
                }))?
            );
        }
        Commands::Doctor(args) => {
            let store = open_store(&args.root, cli.store.as_deref())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "ready",
                    "store_path": store.path(),
                    "observation_count": store.observation_count()?,
                    "decision_count": store.decision_count()?,
                    "contract": crate::model::CONTRACT_VERSION,
                    "weight_profile": crate::profiles::PROFILE_VERSION
                }))?
            );
        }
    }
    Ok(())
}

fn open_store(root: &Path, configured: Option<&Path>) -> Result<Store> {
    let store_path = match configured {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => root.join(path),
        None => root.join(DEFAULT_STORE_REL_PATH),
    };
    Store::open(&store_path)
}

fn review_state(value: ReviewStateArg) -> OperatorReviewState {
    match value {
        ReviewStateArg::Accepted => OperatorReviewState::Accepted,
        ReviewStateArg::Rejected => OperatorReviewState::Rejected,
        ReviewStateArg::Deferred => OperatorReviewState::Deferred,
        ReviewStateArg::NeedsFollowUp => OperatorReviewState::NeedsFollowUp,
        ReviewStateArg::HistoricalOnly => OperatorReviewState::HistoricalOnly,
    }
}

#[allow(dead_code)]
fn require_existing(path: &Path) -> Result<()> {
    if path.exists() {
        Ok(())
    } else {
        Err(WormError::Config(format!(
            "{} does not exist",
            path.display()
        )))
    }
}
