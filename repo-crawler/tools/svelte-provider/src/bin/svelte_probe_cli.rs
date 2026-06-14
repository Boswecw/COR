use std::path::PathBuf;
use std::process::Stdio;

use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Parser, Debug)]
#[command(name = "svelte-probe-cli")]
#[command(about = "Run the Bun/Svelte probe against a .svelte file and return JSON")]
struct Args {
    /// Path to the .svelte file to probe
    file: PathBuf,

    /// Path to the provider directory, relative to the repo root by default
    #[arg(long, default_value = "tools/svelte-provider")]
    provider_dir: PathBuf,

    /// Script path inside the provider directory
    #[arg(long, default_value = "src/probe.ts")]
    script: PathBuf,

    /// Print the raw probe stdout exactly as returned
    #[arg(long, default_value_t = false)]
    raw: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProbeParsedWith {
    engine: Option<String>,
    mode: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProbeRunes {
    state: Option<bool>,
    derived: Option<bool>,
    effect: Option<bool>,
    props: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProbeTemplate {
    snippet: Option<bool>,
    render: Option<bool>,
    #[serde(rename = "legacyEventDirective")]
    legacy_event_directive: Option<bool>,
    #[serde(rename = "eventAttributes")]
    event_attributes: Option<bool>,
    #[serde(rename = "styleBlock")]
    style_block: Option<bool>,
    #[serde(rename = "scriptInstance")]
    script_instance: Option<bool>,
    #[serde(rename = "scriptModule")]
    script_module: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProbeHeuristics {
    #[serde(rename = "hasSvelte5Runes")]
    has_svelte5_runes: Option<bool>,
    runes: Option<ProbeRunes>,
    template: Option<ProbeTemplate>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProbeError {
    kind: Option<String>,
    message: Option<String>,
    code: Option<String>,
    details: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProbeResponse {
    ok: bool,
    provider: Option<String>,
    version: Option<u32>,
    file: Option<String>,
    #[serde(rename = "absolutePath")]
    absolute_path: Option<String>,
    ext: Option<String>,
    bytes: Option<u64>,
    sha256: Option<String>,
    #[serde(rename = "parsedWith")]
    parsed_with: Option<ProbeParsedWith>,
    heuristics: Option<ProbeHeuristics>,
    error: Option<ProbeError>,
}

fn resolve_from_cwd(path: PathBuf) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(std::env::current_dir()
            .context("failed to resolve current working directory")?
            .join(path))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let input_file = resolve_from_cwd(args.file)?
        .canonicalize()
        .context("failed to resolve target .svelte file")?;

    if !input_file.is_file() {
        bail!("target is not a file: {}", input_file.display());
    }

    let provider_dir = resolve_from_cwd(args.provider_dir)?;
    if !provider_dir.is_dir() {
        bail!("provider_dir does not exist: {}", provider_dir.display());
    }

    let script_path = provider_dir.join(&args.script);
    if !script_path.is_file() {
        bail!("probe script does not exist: {}", script_path.display());
    }

    let output = Command::new("bun")
        .arg("run")
        .arg(&args.script)
        .arg(&input_file)
        .current_dir(&provider_dir)
        .stdin(Stdio::null())
        .output()
        .await
        .context("failed to execute bun probe")?;

    let stdout = String::from_utf8(output.stdout).context("probe stdout was not valid UTF-8")?;
    let stderr = String::from_utf8(output.stderr).context("probe stderr was not valid UTF-8")?;

    if stdout.trim().is_empty() {
        if !stderr.trim().is_empty() {
            eprintln!("{stderr}");
        }
        bail!("probe returned empty stdout");
    }

    if args.raw {
        println!("{stdout}");
    }

    let parsed: ProbeResponse =
        serde_json::from_str(&stdout).context("failed to parse probe JSON output")?;

    if !output.status.success() || !parsed.ok {
        if !args.raw {
            println!("{}", serde_json::to_string_pretty(&parsed)?);
        }

        if !stderr.trim().is_empty() {
            eprintln!("{stderr}");
        }

        let message = parsed
            .error
            .as_ref()
            .and_then(|e| e.message.clone())
            .unwrap_or_else(|| "probe reported failure".to_string());

        bail!("{message}");
    }

    if !args.raw {
        println!("{}", serde_json::to_string_pretty(&parsed)?);
    }

    Ok(())
}