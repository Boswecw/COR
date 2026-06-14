use anyhow::Result;

#[path = "../svelte_probe.rs"]
mod svelte_probe;

use svelte_probe::{run_self_check, SvelteProbeConfig, DEFAULT_TIMEOUT_MS};

#[tokio::main]
async fn main() -> Result<()> {
    let config = SvelteProbeConfig::from_current_dir(
        "tools/svelte-provider".into(),
        "src/probe.ts".into(),
        DEFAULT_TIMEOUT_MS,
    )?;

    let execution = run_self_check(&config).await?;
    println!("{}", serde_json::to_string_pretty(&execution.response)?);

    Ok(())
}
