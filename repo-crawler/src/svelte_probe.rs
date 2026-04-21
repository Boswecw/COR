use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub const EXPECTED_PROVIDER: &str = "svelte-probe";
pub const EXPECTED_VERSION: u32 = 1;
pub const DEFAULT_TIMEOUT_MS: u64 = 10_000;
pub const MAX_STDOUT_BYTES: usize = 128 * 1024;
pub const MAX_STDERR_BYTES: usize = 128 * 1024;

#[derive(Debug, Clone)]
pub struct SvelteProbeConfig {
    pub repo_root: PathBuf,
    pub provider_dir: PathBuf,
    pub script: PathBuf,
    pub timeout_ms: u64,
}

impl SvelteProbeConfig {
    pub fn from_current_dir(provider_dir: PathBuf, script: PathBuf, timeout_ms: u64) -> Result<Self> {
        let repo_root = canonicalize_with_context(
            &std::env::current_dir().context("failed to resolve repo root")?,
            "repo root",
        )?;

        let provider_dir = canonicalize_with_context(&resolve_from_cwd(provider_dir)?, "provider_dir")?;
        if !provider_dir.is_dir() {
            bail!("provider_dir does not exist: {}", provider_dir.display());
        }
        assert_within_repo(&repo_root, &provider_dir, "provider_dir")?;

        let script_path = provider_dir.join(&script);
        if !script_path.is_file() {
            bail!("probe script does not exist: {}", script_path.display());
        }

        Ok(Self {
            repo_root,
            provider_dir,
            script,
            timeout_ms,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProbeParsedWith {
    pub engine: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProbeRunes {
    pub state: Option<bool>,
    pub derived: Option<bool>,
    pub effect: Option<bool>,
    pub props: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProbeTemplate {
    pub snippet: Option<bool>,
    pub render: Option<bool>,
    #[serde(rename = "legacyEventDirective")]
    pub legacy_event_directive: Option<bool>,
    #[serde(rename = "eventAttributes")]
    pub event_attributes: Option<bool>,
    #[serde(rename = "styleBlock")]
    pub style_block: Option<bool>,
    #[serde(rename = "scriptInstance")]
    pub script_instance: Option<bool>,
    #[serde(rename = "scriptModule")]
    pub script_module: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProbeHeuristics {
    #[serde(rename = "hasSvelte5Runes")]
    pub has_svelte5_runes: Option<bool>,
    pub runes: Option<ProbeRunes>,
    pub template: Option<ProbeTemplate>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProbeError {
    pub kind: Option<String>,
    pub message: Option<String>,
    pub code: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProbeRuntime {
    pub node: Option<String>,
    pub bun: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProbeLimits {
    #[serde(rename = "maxFileBytes")]
    pub max_file_bytes: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProbeResponse {
    pub ok: bool,
    pub provider: String,
    pub version: u32,
    pub mode: Option<String>,
    pub runtime: Option<ProbeRuntime>,
    pub limits: Option<ProbeLimits>,
    pub file: Option<String>,
    #[serde(rename = "absolutePath")]
    pub absolute_path: Option<String>,
    pub ext: Option<String>,
    pub bytes: Option<u64>,
    pub sha256: Option<String>,
    #[serde(rename = "parsedWith")]
    pub parsed_with: Option<ProbeParsedWith>,
    pub heuristics: Option<ProbeHeuristics>,
    pub error: Option<ProbeError>,
}

#[derive(Debug, Clone)]
pub struct SvelteProbeExecution {
    pub response: ProbeResponse,
    pub stdout: String,
    pub stderr: String,
    pub exit_success: bool,
}

pub async fn run_self_check(config: &SvelteProbeConfig) -> Result<SvelteProbeExecution> {
    run_provider_command(config, [OsString::from(&config.script), OsString::from("--self-check")]).await
}

pub async fn run_probe(config: &SvelteProbeConfig, file: PathBuf) -> Result<SvelteProbeExecution> {
    let input_file = canonicalize_with_context(&resolve_from_cwd(file)?, "target .svelte file")?;
    if !input_file.is_file() {
        bail!("target is not a file: {}", input_file.display());
    }
    if input_file.extension().and_then(|s| s.to_str()) != Some("svelte") {
        bail!("target file must have .svelte extension: {}", input_file.display());
    }
    assert_within_repo(&config.repo_root, &input_file, "target file")?;

    run_provider_command(
        config,
        [OsString::from(&config.script), input_file.into_os_string()],
    )
    .await
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

fn canonicalize_with_context(path: &Path, label: &str) -> Result<PathBuf> {
    path.canonicalize()
        .with_context(|| format!("failed to canonicalize {label}: {}", path.display()))
}

fn assert_within_repo(repo_root: &Path, path: &Path, label: &str) -> Result<()> {
    if !path.starts_with(repo_root) {
        bail!("{label} is outside repo root: {}", path.display());
    }
    Ok(())
}

fn validate_contract(parsed: &ProbeResponse) -> Result<()> {
    if parsed.provider != EXPECTED_PROVIDER {
        bail!(
            "provider contract mismatch: expected {}, got {}",
            EXPECTED_PROVIDER,
            parsed.provider
        );
    }

    if parsed.version != EXPECTED_VERSION {
        bail!(
            "provider contract mismatch: expected version {}, got {}",
            EXPECTED_VERSION,
            parsed.version
        );
    }

    Ok(())
}

async fn run_provider_command<I>(config: &SvelteProbeConfig, args: I) -> Result<SvelteProbeExecution>
where
    I: IntoIterator<Item = OsString>,
{
    let child = Command::new("bun")
        .arg("run")
        .args(args)
        .current_dir(&config.provider_dir)
        .stdin(Stdio::null())
        .output();

    let output = timeout(Duration::from_millis(config.timeout_ms), child)
        .await
        .with_context(|| format!("probe timed out after {} ms", config.timeout_ms))?
        .context("failed to execute bun probe")?;

    if output.stdout.len() > MAX_STDOUT_BYTES {
        bail!(
            "probe stdout exceeded {} bytes (actual {})",
            MAX_STDOUT_BYTES,
            output.stdout.len()
        );
    }

    if output.stderr.len() > MAX_STDERR_BYTES {
        bail!(
            "probe stderr exceeded {} bytes (actual {})",
            MAX_STDERR_BYTES,
            output.stderr.len()
        );
    }

    let stdout = String::from_utf8(output.stdout).context("probe stdout was not valid UTF-8")?;
    let stderr = String::from_utf8(output.stderr).context("probe stderr was not valid UTF-8")?;

    if stdout.trim().is_empty() {
        if !stderr.trim().is_empty() {
            eprintln!("{stderr}");
        }
        bail!("probe returned empty stdout");
    }

    let parsed: ProbeResponse =
        serde_json::from_str(&stdout).context("failed to parse probe JSON output")?;

    validate_contract(&parsed)?;

    Ok(SvelteProbeExecution {
        response: parsed,
        stdout,
        stderr,
        exit_success: output.status.success(),
    })
}
