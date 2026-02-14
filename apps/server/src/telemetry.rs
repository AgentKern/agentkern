use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator};
use tracing_subscriber::{Registry, layer::SubscriberExt, util::SubscriberInitExt};

use crate::auth::Environment;

pub fn init_telemetry() -> anyhow::Result<()> {
    // Set global propagator for context propagation
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Configure OTLP exporter (Jaeger/Honeycomb)
    // Default endpoint: http://localhost:4317 (gRPC)
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()?;

    // Configure tracer provider
    let provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_resource(Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", "agentkern-server"),
            opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            opentelemetry::KeyValue::new(
                "deployment.environment",
                match Environment::from_env() {
                    Environment::Development => "development",
                    Environment::Staging => "staging",
                    Environment::Production => "production",
                },
            ),
        ]))
        .build();

    global::set_tracer_provider(provider.clone());
    let tracer = provider.tracer("agentkern-server");

    // Create tracing layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Standard env filter (RUST_LOG)
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,agentkern_server=debug,tower_http=debug".into());

    // Format layer for stdout (keep logs readable)
    let formatting_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .compact();

    // Initialize subscriber with both layers
    Registry::default()
        .with(env_filter)
        .with(telemetry)
        .with(formatting_layer)
        .try_init()?;

    Ok(())
}

#[allow(dead_code)]
pub fn shutdown_telemetry() {
    global::shutdown_tracer_provider();
}
