//! Mixpanel tracking client - telemetry removed from this build.
//!
//! This build ships without any analytics capability. The public API
//! surface is preserved so callers compile, but `track` and `engage` are
//! no-ops: no HTTP request is ever made and no endpoint URL exists in
//! this crate.

use std::collections::HashMap;

/// Former Mixpanel client. In this build it holds no HTTP client and
/// performs no network I/O.
#[derive(Clone)]
pub struct Mixpanel {
    #[allow(dead_code)]
    token: String,
}

/// Error type for Mixpanel operations. Retained for API compatibility;
/// never produced in this build.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("JSON serialization failed: {0}")]
    Json(#[from] serde_json::Error),
}

impl Mixpanel {
    /// Create a new client handle. Telemetry removed from this build: the
    /// token is retained only for signature compatibility.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }

    /// Create a new client handle. The reqwest client argument is accepted
    /// for signature compatibility and dropped - no requests are made.
    pub fn with_client(token: impl Into<String>, _client: reqwest::Client) -> Self {
        Self::new(token)
    }

    /// Telemetry removed from this build: no-op, sends nothing.
    pub async fn track(
        &self,
        _event: &str,
        _properties: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// Telemetry removed from this build: no-op, sends nothing.
    pub async fn engage(
        &self,
        _distinct_id: &str,
        _set: HashMap<String, serde_json::Value>,
    ) -> Result<(), Error> {
        Ok(())
    }
}
