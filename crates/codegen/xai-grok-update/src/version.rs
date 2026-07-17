//! Version checking - update phone-home removed from this build.
//!
//! The former module fetched release channel pointers and registry versions
//! over the network. This build ships without any update check: the version
//! fetch functions return an error immediately and perform no network I/O
//! and spawn no processes. Local, offline functionality (the version cache
//! file, channel label derivation, and on-disk version probing) is kept so
//! `--version` display keeps working.

use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;
use tokio::fs;

use xai_grok_shell::env::GrokBuildEnvironment;
use xai_grok_shell::util::grok_home::grok_home;

const TTL_SECONDS_BEFORE_AUTO_UPDATE: Duration = Duration::from_secs(60 * 30);

pub(crate) const UPDATE_CHECKS_DISABLED: &str = "update checks disabled in this build";

/// Minimal configuration the update system needs from the environment.
/// Retained for API compatibility; no field is ever used for network I/O in
/// this build.
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    /// Chat API proxy base URL. Retained for signature compatibility.
    pub proxy_base_url: String,
    /// Auth scope key for `~/.grok/auth.json`.
    pub auth_scope: String,
    /// Enterprise deployment key (GROK_DEPLOYMENT_KEY).
    pub deployment_key: Option<String>,
    /// Optional extra auth material. Retained for signature compatibility.
    pub alpha_test_key: Option<String>,
    /// Release channel: "stable" or "alpha". Loaded from config.
    pub channel: String,
    /// Custom npm registry URL. Retained for signature compatibility.
    pub npm_registry: Option<String>,
}

impl UpdateConfig {
    pub fn from_environment(env: &GrokBuildEnvironment) -> Self {
        Self {
            proxy_base_url: env.cli_chat_proxy_base_url(),
            auth_scope: xai_grok_shell::auth::GrokComConfig::default().auth_scope(),
            deployment_key: None,
            alpha_test_key: None,
            channel: "stable".to_string(),
            npm_registry: None,
        }
    }
}

#[derive(Debug, serde::Serialize, Deserialize)]
struct GrokVersion {
    version: String,
    #[serde(default)]
    stable_version: Option<String>,
    checked_at: String,
}

impl GrokVersion {
    fn is_fresh(&self, now: time::OffsetDateTime, ttl: Duration) -> bool {
        if let Ok(dt) = time::OffsetDateTime::parse(
            &self.checked_at,
            &time::format_description::well_known::Rfc3339,
        ) {
            // Clock-skew guard: future timestamps are never fresh.
            if dt > now {
                return false;
            }
            now - dt < ttl
        } else {
            false
        }
    }

    fn new(version: String, stable_version: Option<String>, now: time::OffsetDateTime) -> Self {
        let checked_at = now
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| now.to_string());
        Self {
            version,
            stable_version,
            checked_at,
        }
    }
}

/// Update checks removed from this build: always returns an error, performs
/// no network I/O and spawns no processes.
pub async fn fetch_latest_version(_installer: &str, _config: &UpdateConfig) -> Result<String> {
    anyhow::bail!(UPDATE_CHECKS_DISABLED)
}

/// Write the version cache to disk (purely local file I/O).
pub async fn write_version_cache(version: &str, stable_version: Option<&str>) {
    let version_path = grok_home().join("version.json");
    let now = time::OffsetDateTime::now_utc();
    let json = GrokVersion::new(
        version.to_string(),
        stable_version.map(|s| s.to_string()),
        now,
    );
    if let Some(dir) = version_path.parent()
        && let Err(e) = fs::create_dir_all(dir).await
    {
        tracing::warn!("failed to create version cache directory: {}", e);
        return;
    }
    let tmp = version_path.with_extension("json.tmp");
    let data = match serde_json::to_vec_pretty(&json) {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("failed to serialize version cache: {}", e);
            return;
        }
    };
    if let Err(e) = fs::write(&tmp, data).await {
        tracing::warn!("failed to write version cache tmp file: {}", e);
        return;
    }
    if let Err(e) = fs::rename(&tmp, &version_path).await {
        tracing::warn!("failed to rename version cache file: {}", e);
    }
}

/// Update checks removed from this build: always returns an error, performs
/// no network I/O.
pub async fn get_latest_version(_installer: &str, _config: &UpdateConfig) -> Result<String> {
    anyhow::bail!(UPDATE_CHECKS_DISABLED)
}

/// True if `version.json` exists and is within TTL (purely local file I/O).
pub async fn is_version_cache_fresh() -> bool {
    let version_path = grok_home().join("version.json");
    let now = time::OffsetDateTime::now_utc();
    if let Ok(version_str) = fs::read_to_string(&version_path).await
        && let Ok(version) = serde_json::from_str::<GrokVersion>(&version_str)
        && version.is_fresh(now, TTL_SECONDS_BEFORE_AUTO_UPDATE)
    {
        return true;
    }
    false
}

pub use xai_grok_version::installed as get_installed_grok_version;

/// Version of the managed grok binary currently on disk, read from the
/// `~/.grok/bin/grok` symlink target without exec'ing anything (purely
/// local).
pub fn installed_on_disk_version() -> Option<String> {
    #[cfg(unix)]
    {
        let app = xai_grok_shell::util::grok_home::grok_application();
        let target = std::fs::read_link(&app).ok()?;
        // metadata() follows the symlink: Err means the target is gone
        // (dangling link) and the version it names is not actually on disk.
        std::fs::metadata(&app).ok()?;
        version_from_versioned_binary_name(target.file_name()?.to_str()?, "grok")
    }
    #[cfg(not(unix))]
    {
        None
    }
}

/// Extract the `<version>` portion of a versioned binary file name (purely
/// local string parsing).
pub(crate) fn version_from_versioned_binary_name(name: &str, bin_prefix: &str) -> Option<String> {
    const PLATFORM_OS: &[&str] = &["macos", "linux", "darwin", "windows"];
    let suffix = name.strip_prefix(bin_prefix)?.strip_prefix('-')?;
    let parts: Vec<&str> = suffix.split('-').collect();
    let platform_start = parts
        .iter()
        .position(|p| PLATFORM_OS.contains(p))
        .unwrap_or(parts.len());
    let ver_str = parts[..platform_start].join("-");
    semver::Version::parse(&ver_str).ok()?;
    Some(ver_str)
}

/// Former stable-pointer fetch. Update checks removed from this build:
/// always `None`, no network I/O.
pub(crate) async fn try_fetch_stable_pointer() -> Option<String> {
    None
}

/// Read the cached stable version from `~/.grok/version.json` (sync, for
/// display; purely local).
pub fn cached_stable_version() -> Option<String> {
    let version_path = grok_home().join("version.json");
    let content = std::fs::read_to_string(&version_path).ok()?;
    let gv: GrokVersion = serde_json::from_str(&content).ok()?;
    gv.stable_version
}

/// Pure comparison: derive the channel name from current vs stable pointer.
fn derive_channel<'a>(current: &str, stable: &str) -> Option<&'a str> {
    let current_v = semver::Version::parse(current).ok()?;
    let stable_v = semver::Version::parse(stable).ok()?;
    if current_v > stable_v {
        Some("alpha")
    } else {
        Some("stable")
    }
}

/// Machine-readable channel name derived from the cached stable pointer
/// (purely local; the cache is only refreshed by older builds).
pub fn channel_name() -> Option<&'static str> {
    use std::sync::OnceLock;
    static NAME: OnceLock<Option<&'static str>> = OnceLock::new();
    *NAME.get_or_init(|| {
        let stable = cached_stable_version()?;
        derive_channel(xai_grok_version::VERSION, &stable)
    })
}

/// Channel label derived from the cached stable pointer (purely local).
pub fn channel_label() -> &'static str {
    use std::sync::OnceLock;
    static LABEL: OnceLock<&'static str> = OnceLock::new();
    LABEL.get_or_init(|| {
        let stable = match cached_stable_version() {
            Some(s) => s,
            None => return "",
        };
        match derive_channel(xai_grok_version::VERSION, &stable) {
            Some("alpha") => " [alpha]",
            Some(_) => " [stable]",
            None => "",
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that a future `checked_at` timestamp (e.g. from clock skew or
    /// NTP time-warp) is never considered fresh.
    #[test]
    fn test_is_fresh_rejects_future_timestamp() {
        let now = time::OffsetDateTime::now_utc();
        let future = now + Duration::from_secs(600);
        let v = GrokVersion::new("0.1.200".to_string(), None, future);
        assert!(
            !v.is_fresh(now, Duration::from_secs(30)),
            "Future timestamp must not be considered fresh (clock-skew guard)."
        );
    }

    /// Disk-version probe: parsing the version out of the managed install's
    /// symlink-target file name (`grok-<version>-<platform>`).
    #[test]
    fn test_version_from_versioned_binary_name() {
        let cases: &[(&str, Option<&str>)] = &[
            ("grok-0.2.46-darwin-arm64", Some("0.2.46")),
            ("grok-0.1.220-linux-x86_64", Some("0.1.220")),
            ("grok-0.2.5-windows-x86_64.exe", Some("0.2.5")),
            ("grok-0.1.220-alpha.4-linux-x86_64", Some("0.1.220-alpha.4")),
            ("grok-0.1.220-alpha.4", Some("0.1.220-alpha.4")),
            ("grok-pager-0.1.5-darwin-arm64", None),
            ("grok-garbage-darwin-arm64", None),
            ("grok-0.2.46", Some("0.2.46")),
            ("other-0.2.46-darwin-arm64", None),
            ("grok-latest", None),
            ("grok", None),
            ("", None),
        ];
        for (name, expected) in cases {
            assert_eq!(
                version_from_versioned_binary_name(name, "grok").as_deref(),
                *expected,
                "version_from_versioned_binary_name({name:?})"
            );
        }

        assert_eq!(
            version_from_versioned_binary_name("grok-pager-0.1.5-darwin-arm64", "grok-pager")
                .as_deref(),
            Some("0.1.5")
        );
    }

    #[test]
    fn test_derive_channel_matrix() {
        let cases: &[(&str, &str, Option<&str>)] = &[
            ("0.1.220-alpha.2", "0.1.219", Some("alpha")),
            ("0.1.219", "0.1.219", Some("stable")),
            ("0.1.218", "0.1.219", Some("stable")),
            ("0.1.220-alpha.2", "0.1.220-alpha.2", Some("stable")),
            ("0.1.220-alpha.2", "0.1.220", Some("stable")),
            ("0.2.5", "0.2.3", Some("alpha")),
            ("0.2.5", "0.2.5", Some("stable")),
            ("0.2.3", "0.2.5", Some("stable")),
            ("0.2.0", "0.2.0", Some("stable")),
            ("0.2.0", "0.1.219", Some("alpha")),
            ("0.1.220-alpha.2", "0.2.0", Some("stable")),
            ("garbage", "0.1.219", None),
            ("0.1.219", "garbage", None),
            ("", "0.1.219", None),
            ("0.1.219", "", None),
        ];

        for (current, stable, expected) in cases {
            let result = derive_channel(current, stable);
            assert_eq!(
                result, *expected,
                "derive_channel({:?}, {:?}) = {:?}, expected {:?}",
                current, stable, result, expected,
            );
        }
    }

    #[test]
    fn test_version_json_backward_compat() {
        // Old format (no stable_version) must parse — serde(default) fills None.
        let old = r#"{"version":"0.1.180","checked_at":"2026-04-22T10:30:00Z"}"#;
        let v: GrokVersion = serde_json::from_str(old).unwrap();
        assert_eq!(v.version, "0.1.180");
        assert!(v.stable_version.is_none());

        // New format with stable_version round-trips correctly.
        let now = time::OffsetDateTime::now_utc();
        let new = GrokVersion::new("0.2.5".to_string(), Some("0.2.3".to_string()), now);
        let json = serde_json::to_string(&new).unwrap();
        let parsed: GrokVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, "0.2.5");
        assert_eq!(parsed.stable_version.as_deref(), Some("0.2.3"));

        assert!(
            time::OffsetDateTime::parse(
                &parsed.checked_at,
                &time::format_description::well_known::Rfc3339,
            )
            .is_ok()
        );

        let future = r#"{"version":"0.1.180","checked_at":"2026-04-22T10:30:00Z","future":"ok"}"#;
        assert!(serde_json::from_str::<GrokVersion>(future).is_ok());

        let missing = r#"{"version":"0.1.180"}"#;
        assert!(serde_json::from_str::<GrokVersion>(missing).is_err());
    }

    #[test]
    fn test_is_fresh_ttl_boundaries() {
        let now = time::OffsetDateTime::now_utc();
        let v = GrokVersion::new("0.1.200".to_string(), None, now);

        assert!(v.is_fresh(now, Duration::from_secs(60)));
        assert!(v.is_fresh(now + Duration::from_secs(29), Duration::from_secs(30)));
        assert!(!v.is_fresh(now + Duration::from_secs(30), Duration::from_secs(30)));
        assert!(!v.is_fresh(now + Duration::from_secs(31), Duration::from_secs(30)));
        assert!(!v.is_fresh(now, Duration::ZERO));

        let bad = GrokVersion {
            version: "0.1.200".to_string(),
            stable_version: None,
            checked_at: "not-rfc3339".to_string(),
        };
        assert!(!bad.is_fresh(now, Duration::from_secs(60)));
    }

    #[test]
    fn test_update_config_default_channel_is_stable() {
        use xai_grok_shell::env::GrokBuildEnvironment;
        let cfg = UpdateConfig::from_environment(&GrokBuildEnvironment::Production);
        assert_eq!(cfg.channel, "stable");
    }
}
