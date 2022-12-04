use std::env;

use color_eyre::Result;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

use opentelemetry::{
    sdk::{trace, Resource},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tonic::{metadata::*, transport::ClientTlsConfig};

pub fn setup() -> Result<()> {
    let mut map = MetadataMap::with_capacity(2);
    let environment = env::var("ENVIRONMENT")?;

    map.insert("x-environment", environment.parse()?);
    map.insert("x-honeycomb-team", env::var("HONEYCOMB_KEY")?.parse()?);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(env::var("EXPORTER_URL")?)
                .with_tls_config(ClientTlsConfig::new())
                .with_metadata(map),
        )
        .with_trace_config(
            trace::config().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                format!("ilquentir-{environment}"),
            )])),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .with(telemetry)
        .init();

    Ok(())
}

pub fn teardown() {
    opentelemetry::global::shutdown_tracer_provider();
}
