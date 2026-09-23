use std::sync::{Arc, OnceLock};

use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::Layer as _;
use tracing_subscriber::registry::LookupSpan;

use crate::config::{OtelClientInfo, OtelLayerConfig};

static TRACER_PROVIDER: OnceLock<SdkTracerProvider> = OnceLock::new();

#[derive(Debug, Clone, Copy)]
pub enum OtelProviderMode {
    Server,
    Local,
}

pub type SessionMetricsGate = Arc<dyn Fn() -> bool + Send + Sync>;

/// Redacts each span batch in place before export. The domain layer injects the allowlist,
/// so this foundation crate holds no product-specific field policy.
pub type SpanRedactor = Arc<dyn Fn(&mut [opentelemetry_sdk::trace::SpanData]) + Send + Sync>;

const ENV_OTEL_FILTER: &str = "GROK_OTEL_FILTER";
const DEFAULT_OTEL_FILTER: &str = "info";

pub fn build_otel_layer<S>(
    client: OtelClientInfo,
    config: OtelLayerConfig,
    mode: OtelProviderMode,
    session_metrics_gate: SessionMetricsGate,
    redact: SpanRedactor,
) -> impl tracing_subscriber::layer::Layer<S>
where
    S: tracing::Subscriber + for<'span> LookupSpan<'span>,
{
    let provider = TRACER_PROVIDER
        .get_or_init(|| build_tracer_provider(client, config, mode, session_metrics_gate, redact));
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

fn build_tracer_provider(
    client: OtelClientInfo,
    _config: OtelLayerConfig,
    mode: OtelProviderMode,
    _session_metrics_gate: SessionMetricsGate,
    _redact: SpanRedactor,
) -> SdkTracerProvider {
    match mode {
        // Span export removed from this build: the former server mode
        // attached an OTLP batch exporter posting every span to a product
        // observability backend. Both modes now build a provider with NO
        // span processor and no exporter - spans are created and dropped
        // locally, and nothing is ever sent anywhere. The signature and the
        // resource stay so the callers compile unchanged.
        OtelProviderMode::Server => SdkTracerProvider::builder()
            .with_resource(crate::config::build_base_resource(client))
            .build(),
        OtelProviderMode::Local => SdkTracerProvider::builder().build(),
    }
}


pub fn shutdown_provider() {
    let Some(provider) = TRACER_PROVIDER.get() else {
        return;
    };
    let started = std::time::Instant::now();
    crate::timeout::run_with_timeout(
        "otel provider",
        crate::timeout::OTEL_SHUTDOWN_TIMEOUT,
        move || {
            if let Err(e) = provider.force_flush() {
                tracing::debug!("[otel] Failed to flush tracer provider: {}", e);
            }
            if let Err(e) = provider.shutdown() {
                tracing::debug!("[otel] Failed to shutdown tracer provider: {}", e);
            }
        },
    );
    tracing::debug!(
        "[otel] provider shutdown took {}ms",
        started.elapsed().as_millis()
    );
}
