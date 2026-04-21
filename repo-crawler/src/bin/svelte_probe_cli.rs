use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;

#[path = "../svelte_probe.rs"]
mod svelte_probe;

use svelte_probe::{run_probe, run_self_check, SvelteProbeConfig, DEFAULT_TIMEOUT_MS};

#[derive(Parser, Debug)]
#[command(name = "svelte-probe-cli")]
#[command(about = "Run the Bun/Svelte probe through the shared Rust module")]
struct Args {
    /// Path to the .svelte file to probe
    file: Option<PathBuf>,

    /// Run the provider self-check instead of probing a file
    #[arg(long, default_value_t = false)]
    self_check: bool,

    /// Path to the provider directory, relative to the repo root by default
    #[arg(long, default_value = "tools/svelte-provider")]
    provider_dir: PathBuf,

    /// Script path inside the provider directory
    #[arg(long, default_value = "src/probe.ts")]
    script: PathBuf,

    /// Timeout in milliseconds for the Bun probe process
    #[arg(long, default_value_t = DEFAULT_TIMEOUT_MS)]
    timeout_ms: u64,

    /// Print the raw provider stdout exactly as returned
    #[arg(long, default_value_t = false)]
    raw: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let config = SvelteProbeConfig::from_current_dir(
        args.provider_dir,
        args.script,
        args.timeout_ms,
    )?;

    let execution = if args.self_check {
        run_self_check(&config).await?
    } else {
        let file = args
            .file
            .clone()
            .ok_or_else(|| anyhow::anyhow!("file path is required unless --self-check is used"))?;
        run_probe(&config, file).await?
    };

    if args.raw {
        println!("{}", execution.stdout);
    } else {
        println!("{}", serde_json::to_string_pretty(&execution.response)?);
    }

    if !execution.exit_success || !execution.response.ok {
        if !execution.stderr.trim().is_empty() {
            eprintln!("{}", execution.stderr);
        }

        let message = execution
            .response
            .error
            .as_ref()
            .and_then(|e| e.message.clone())
            .unwrap_or_else(|| "probe reported failure".to_string());

        bail!("{message}");
    }

    Ok(())
}
