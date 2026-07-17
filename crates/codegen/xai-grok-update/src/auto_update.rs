//! Auto-update - phone-home removed from this build.
//!
//! The former module checked release channel pointers, downloaded binaries,
//! and ran install scripts. This build ships without any of that: update
//! checks return "nothing available" immediately, install entry points
//! return an error, and no network I/O happens and no installer process is
//! spawned. Local functionality that other code depends on (installer
//! detection from env/config, restart-onto-installed-binary, channel
//! switching in config) is kept - none of it touches the network.

use anyhow::Result;
use std::process::Command;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use crate::version::{UPDATE_CHECKS_DISABLED, UpdateConfig, get_installed_grok_version};
use xai_grok_shell::util::config;
use xai_grok_shell::util::grok_home::grok_application;

#[derive(Clone, Copy, Debug)]
pub enum UpdateRunMode {
    Blocking,
    NonBlocking,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub installer: Option<String>,
    pub channel: String,
    pub auto_update: Option<bool>,
    pub error: Option<String>,
}

/// Format and print an [`UpdateStatus`] to stdout.
pub fn print_update_status(status: &UpdateStatus, json: bool) -> anyhow::Result<()> {
    if json {
        let payload = serde_json::to_string(status)?;
        println!("{payload}");
        return Ok(());
    }

    if let Some(error) = status.error.as_deref() {
        println!(
            "Your Own AI Build - v{} [{}]",
            status.current_version, status.channel
        );
        println!("Update check skipped: {error}");
        return Ok(());
    }

    let channel_label = format!(" [{}]", status.channel);

    if status.update_available {
        if let Some(latest_version) = status.latest_version.as_deref() {
            println!(
                "A new version of Your Own AI Build is available: {} -> {}{}",
                status.current_version, latest_version, channel_label
            );
        } else {
            println!("A new version of Your Own AI Build is available.");
        }
        return Ok(());
    }

    if let Some(latest_version) = status.latest_version.as_deref() {
        println!(
            "Your Own AI Build - v{} (latest: {}){}",
            status.current_version, latest_version, channel_label
        );
        return Ok(());
    }

    println!(
        "Your Own AI Build - v{}{}",
        status.current_version, channel_label
    );
    Ok(())
}

/// Update checks removed from this build: reports the local version only,
/// with no network I/O.
pub async fn check_update_status(update_config: &UpdateConfig) -> UpdateStatus {
    let installer = get_installer().await.map(|value| value.to_string());
    let current_version = get_installed_grok_version();
    let current_config = config::load_config().await;
    let auto_update = current_config.cli.auto_update;
    let channel = update_config.channel.clone();

    UpdateStatus {
        current_version,
        latest_version: None,
        update_available: false,
        installer,
        channel,
        auto_update,
        error: Some(UPDATE_CHECKS_DISABLED.to_string()),
    }
}

/// Update checks removed from this build: always `None`, no network I/O.
pub async fn auto_update_target(_update_config: &UpdateConfig) -> Option<(&'static str, String)> {
    None
}

/// Outcome of [`ensure_latest_on_disk`].
#[derive(Debug)]
pub struct EnsureLatestOutcome {
    /// Always `None` in this build (nothing is ever downloaded).
    pub installed: Option<String>,
    /// Always `false` in this build.
    pub relaunch_needed: bool,
}

/// Update checks removed from this build: no-op, no network I/O.
pub async fn ensure_latest_on_disk(_update_config: &UpdateConfig) -> Result<EnsureLatestOutcome> {
    Ok(EnsureLatestOutcome {
        installed: None,
        relaunch_needed: false,
    })
}

fn env_installer() -> Option<&'static str> {
    if let Ok(v) = std::env::var("GROK_INSTALLER") {
        return match v.to_ascii_lowercase().as_str() {
            "npm" => Some("npm"),
            "internal" => Some("internal"),
            "gh-release" | "gh" => Some("gh-release"),
            _ => None,
        };
    }
    if std::env::var_os("GROK_MANAGED_BY_NPM").is_some() {
        return Some("npm");
    }
    if std::env::var_os("GROK_MANAGED_BY_INTERNAL").is_some() {
        return Some("internal");
    }
    if std::env::var_os("npm_config_user_agent").is_some() {
        return Some("npm");
    }
    None
}

/// Detect how this binary was installed (env vars + local config only).
pub async fn get_installer() -> Option<&'static str> {
    if let Some(i) = env_installer() {
        return Some(i);
    }
    let cfg = config::load_config().await;
    match cfg.cli.installer.as_deref() {
        Some("npm") => Some("npm"),
        Some("gh-release") => Some("gh-release"),
        _ => Some("internal"),
    }
}

/// Result of a background update availability check.
#[derive(Debug, Clone)]
pub struct UpdateAvailable {
    /// The latest version string (e.g. "0.1.200").
    pub latest_version: String,
}

/// Outcome of [`check_update_background`].
pub struct BackgroundUpdateCheck {
    /// Always `None` in this build (update checks removed).
    pub update: Option<UpdateAvailable>,
    /// Always `None` in this build (nothing is downloaded).
    pub download: Option<tokio::process::Child>,
}

impl BackgroundUpdateCheck {
    fn none() -> Self {
        Self {
            update: None,
            download: None,
        }
    }
}

/// Update checks removed from this build: always reports no update, with no
/// network I/O and no spawned process.
pub async fn check_update_background(_update_config: &UpdateConfig) -> BackgroundUpdateCheck {
    BackgroundUpdateCheck::none()
}

/// Update checks removed from this build: always `Ok(false)`, with no
/// network I/O and no spawned process.
pub async fn run_update_if_available(
    _run_mode: UpdateRunMode,
    _interactive: bool,
    _update_config: &UpdateConfig,
) -> Result<bool> {
    Ok(false)
}

/// Resolve the grok binary path for re-execution (purely local).
fn resolve_restart_exe() -> Result<std::path::PathBuf> {
    let canonical = grok_application();
    if canonical.exists() {
        return Ok(canonical);
    }
    Ok(std::env::current_exe()?)
}

/// Restart grok with the original command-line arguments (purely local
/// process re-exec; nothing is downloaded).
pub fn restart_grok() -> Result<()> {
    let exe = resolve_restart_exe()?;
    let mut cmd = Command::new(exe);
    for arg in std::env::args_os().skip(1) {
        cmd.arg(arg);
    }
    cmd.env_clear();
    cmd.envs(std::env::vars_os().filter(|(k, _)| k != "GROK_AUTO_UPDATE"));
    eprintln!("Restarting Grok...");

    #[cfg(unix)]
    {
        let err = cmd.exec();
        Err(anyhow::anyhow!("failed to exec for restart: {err}"))
    }
    #[cfg(not(unix))]
    {
        let status = cmd.status()?;
        std::process::exit(status.code().unwrap_or(0));
    }
}

/// Former installer entry point. Update installation removed from this
/// build: always returns an error; downloads nothing and spawns nothing.
pub async fn run_install_script(
    _installer: &str,
    _target: Option<&str>,
    _update_config: &UpdateConfig,
) -> Result<()> {
    anyhow::bail!(UPDATE_CHECKS_DISABLED)
}

/// Persist a channel switch to local config (purely local; the channel is
/// only used for display in this build).
pub async fn apply_channel_switch(channel_switch: Option<&str>, update_config: &mut UpdateConfig) {
    if let Some(ch) = channel_switch
        && update_config.channel != ch
    {
        let _ = config::update_config(|st| {
            st.cli.channel = Some(ch.to_string());
        })
        .await;
        update_config.channel = ch.to_string();
        eprintln!("Switched to {} channel.", ch);
    }
}

/// Former `grok update` implementation. Update installation removed from
/// this build: prints a notice and returns `Ok(None)`; downloads nothing.
pub async fn run_update(
    _force: bool,
    _pinned_version: Option<&str>,
    channel_switch: Option<&str>,
    update_config: &mut UpdateConfig,
) -> Result<Option<String>> {
    apply_channel_switch(channel_switch, update_config).await;
    eprintln!("Self-update is not available in this build ({UPDATE_CHECKS_DISABLED}).");
    eprintln!("Install a newer release manually from your distribution source.");
    Ok(None)
}
