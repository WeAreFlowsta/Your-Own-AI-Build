//! OpenTelemetry tracing layer - span export removed from this build.
//!
//! The former layer exported session spans over OTLP to a product
//! observability backend. This build ships without any span export: the
//! layer still bridges `tracing` spans into an OpenTelemetry
//! `SdkTracerProvider`, but that provider has **no span processor and no
//! exporter** - spans are created and dropped locally, and nothing is ever
//! sent anywhere. The public API surface is preserved so binaries compile.
use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::sync::{Arc, OnceLock};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::Layer as _;
use tracing_subscriber::registry::LookupSpan;
use xai_grok_auth::AuthCredentialProvider;
static TRACER_PROVIDER: OnceLock<SdkTracerProvider> = OnceLock::new();
const ENV_OTEL_FILTER: &str = "GROK_OTEL_FILTER";
const DEFAULT_OTEL_FILTER: &str = "info";
/// Configuration for [`build_otel_layer`]. Retained for API compatibility;
/// the credentials and exporter settings are never used in this build
/// (no exporter is constructed).
pub struct OtelLayerConfig {
    /// Retained for signature compatibility; never read (nothing exports).
    pub credentials: Arc<dyn AuthCredentialProvider>,
    /// Retained for signature compatibility; never sent (nothing exports).
    pub token_header_value: String,
    /// Retained for signature compatibility; never sent (nothing exports).
    pub alpha_test_key: Option<String>,
    pub exporter: OtelExporterConfig,
}
/// Static identity of the client. Retained for API compatibility.
#[derive(Debug, Clone, Copy)]
pub struct OtelClientInfo {
    pub client_name: &'static str,
    pub client_version: &'static str,
    pub service_version: &'static str,
    pub app_entrypoint: &'static str,
}
/// OTLP trace-export transport settings. Retained for API compatibility;
/// ignored in this build (no exporter is constructed).
#[derive(Debug, Default, Clone)]
pub struct OtelExporterConfig {
    pub traces_url: String,
    pub extra_headers: Vec<(String, String)>,
    pub export_interval: Option<std::time::Duration>,
    pub timeout: Option<std::time::Duration>,
    pub enabled: bool,
}
/// Creates an OpenTelemetry layer that bridges tracing spans to OpenTelemetry.
///
/// Span export removed from this build: the tracer provider is built with no
/// span processor, so spans are created for local context propagation only
/// and are never exported anywhere.
pub fn build_otel_layer<S>(
    _client: OtelClientInfo,
    _config: OtelLayerConfig,
) -> impl tracing_subscriber::layer::Layer<S>
where
    S: tracing::Subscriber + for<'span> LookupSpan<'span>,
{
    let provider = TRACER_PROVIDER.get_or_init(|| SdkTracerProvider::builder().build());
    let tracer = provider.tracer("grok-cli");
    global::set_tracer_provider(provider.clone());
    global::set_text_map_propagator(opentelemetry_sdk::propagation::TraceContextPropagator::new());
    let otel_filter =
        std::env::var(ENV_OTEL_FILTER).unwrap_or_else(|_| DEFAULT_OTEL_FILTER.to_string());
    let otel_filter = tracing_subscriber::filter::EnvFilter::try_new(&otel_filter)
        .unwrap_or_else(|e| {
            eprintln!(
                "[otel] Invalid GROK_OTEL_FILTER '{}': {}. Using default '{}'.",
                otel_filter, e, DEFAULT_OTEL_FILTER
            );
            tracing_subscriber::filter::EnvFilter::try_new(DEFAULT_OTEL_FILTER)
                .expect("default otel filter must parse")
        })
        .add_directive(
            "sampling_log=off"
                .parse()
                .expect("static directive must parse"),
        );
    OpenTelemetryLayer::new(tracer)
        .with_context_activation(false)
        .with_filter(otel_filter)
}
/// Flush and shut down the global tracer provider (and the external OTEL
/// stream). Nothing is exported in this build; this only releases local
/// resources. Safe to call multiple times.
pub fn shutdown_otel() {
    crate::external::shutdown();
    if let Some(provider) = TRACER_PROVIDER.get()
        && let Err(e) = provider.shutdown()
    {
        tracing::debug!("[otel] Failed to shutdown tracer provider: {}", e);
    }
}
/// RAII guard that calls [`shutdown_otel`] on drop.
pub struct OtelGuard;
impl Drop for OtelGuard {
    fn drop(&mut self) {
        shutdown_otel();
    }
}
/// Create an [`OtelGuard`].
pub fn otel_guard() -> OtelGuard {
    OtelGuard
}
