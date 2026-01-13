use opentelemetry::global;
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace as sdktrace, Resource};
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};

pub fn init_telemetry() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // Set global propagator for context propagation
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Configure OTLP exporter (Jaeger/Honeycomb)
    // Default endpoint: http://localhost:4317 (gRPC)
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://localhost:4317");

    // Configure tracer provider
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            sdktrace::config().with_resource(Resource::new(vec![
                opentelemetry::KeyValue::new("service.name", "agentkern-server"),
                opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                opentelemetry::KeyValue::new("deployment.environment", std::env::var("AGENTKERN_ENV").unwrap_or_else(|_| "development".into())),
            ])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

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

pub fn shutdown_telemetry() {
    global::shutdown_tracer_provider();
}
