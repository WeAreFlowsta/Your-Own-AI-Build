//! Sentry error reporting - removed from this build.
//!
//! [`init`] always returns a no-op guard with no DSN configured, so no
//! event, panic report, or trace is ever sent anywhere.
use std::sync::OnceLock;
use std::time::Duration;

use sentry::ClientInitGuard;
use sentry::ClientOptions;

const FLUSH_TIMEOUT: Duration = Duration::from_secs(2);

// ─── Host integration ─────────────────────────────────────────────────────

/// Per-host config; everything that varies between binaries lives here.
pub struct Config {
    /// Sentry tag `client`, e.g. `"grok-pager"`.
    pub client: &'static str,
    pub client_version: &'static str,
    pub release: &'static str,
    /// When `true`, [`init`] returns a no-op guard regardless of `SENTRY_DSN`.
    pub disabled: bool,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

// ─── Public API ────────────────────────────────────────────────────────────

/// Former Sentry init. Error reporting removed from this build: always
/// returns a no-op guard with no DSN configured, so no event, panic report,
/// or trace is ever sent anywhere.
pub fn init(config: Config) -> ClientInitGuard {
    let _ = CONFIG.get_or_init(|| config);
    sentry::init(ClientOptions::default())
}

/// Flush in-flight events. Call before `std::process::exit` in signal handlers.
pub fn flush_on_shutdown() {
    if let Some(client) = sentry::Hub::current().client() {
        client.flush(Some(FLUSH_TIMEOUT));
    }
}

