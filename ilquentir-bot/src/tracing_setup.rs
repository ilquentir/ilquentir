use color_eyre::Result;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

use opentelemetry::{
    sdk::{trace, Resource},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tonic::{metadata::*, transport::ClientTlsConfig};

use ilquentir_config::Config;

pub fn setup(config: &Config) -> Result<()> {
    let mut map = MetadataMap::with_capacity(2);
    let environment = &config.environment;

    map.insert("x-environment", environment.parse()?);
    map.insert("x-honeycomb-team", config.honeycomb_key.parse()?);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&config.exporter_url)
                .with_tls_config(ClientTlsConfig::new())
                .with_metadata(map),
        )
        .with_trace_config(
            trace::config()
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    format!("ilquentir-{environment}"),
                )]))
                .with_span_limits(trace::SpanLimits {
                    max_events_per_span: 1024,
                    ..Default::default()
                }),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_indent_lines(true)
                .with_bracketed_fields(true)
                .with_thread_names(true)
                .with_thread_ids(true),
        )
        .with(telemetry)
        .init();

    Ok(())
}

pub fn teardown() {
    opentelemetry::global::shutdown_tracer_provider();
}
