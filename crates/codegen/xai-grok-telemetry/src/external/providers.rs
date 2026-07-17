//! Provider construction for the external stream - export removed from this
//! build.
//!
//! The former module built OTLP (http/protobuf or gRPC) and console
//! exporters for the opt-in external telemetry stream. This build ships
//! without any telemetry export: `build` always returns empty providers, so
//! the external stream never activates and nothing is ever sent anywhere.

use std::sync::Arc;

use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;

use super::config::ExternalOtelConfig;
use super::redact::{ExportHealth, SharedGates};

pub(crate) struct BuiltProviders {
    pub logger_provider: Option<SdkLoggerProvider>,
    pub meter_provider: Option<SdkMeterProvider>,
}

/// Telemetry export removed from this build: always returns empty providers,
/// so the external stream stays dormant regardless of configuration.
pub(crate) fn build(
    _cfg: &ExternalOtelConfig,
    _gates: SharedGates,
    _health: Arc<ExportHealth>,
) -> Result<BuiltProviders, String> {
    tracing::debug!("external otel: telemetry export removed from this build; stream disabled");
    Ok(BuiltProviders {
        logger_provider: None,
        meter_provider: None,
    })
}
