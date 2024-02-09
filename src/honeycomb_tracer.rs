use opentelemetry_otlp::TonicExporterBuilder;
use opentelemetry_sdk::trace as sdktrace;
use tracing_subscriber::prelude::*;

pub fn honeycomb_tracer(config: sdktrace::Config, exporter: TonicExporterBuilder) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let otlp: sdktrace::Tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(config)
        .with_exporter(exporter)
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("Error: creating otlp tracer");

    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(otlp);

    let subscriber = tracing_subscriber::Registry::default()
        .with(env_filter)
        .with(telemetry_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Error setting global default subscriber");
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::honeycomb_tracer;
    use opentelemetry::trace::TraceContextExt;
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::trace as sdktrace;
    use tonic::metadata::MetadataMap;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    #[tokio::test]
    async fn honeycomb_tracer_generates_nonzero_trace_id() {
        let honeycomb_key = env::var("HONEYCOMB_KEY").unwrap();
        let mut oltp_meta = MetadataMap::new();
        oltp_meta.insert("x-honeycomb-team", honeycomb_key.parse().unwrap());
        oltp_meta.insert("x-honeycomb-dataset", "testdataset".parse().unwrap());

        let config: sdktrace::Config =
            opentelemetry_sdk::trace::config().with_resource(opentelemetry_sdk::Resource::new(
                vec![opentelemetry::KeyValue::new("service.name", "testdataset")],
            ));

        let exporter: opentelemetry_otlp::TonicExporterBuilder = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint("https://api.honeycomb.io:443")
            .with_metadata(oltp_meta);

        honeycomb_tracer(config, exporter);

        let span = tracing::info_span!("test_span");
        let _enter = span.enter();

        let current_span = tracing::Span::current();
        let current_context = current_span.context();
        let spanctx = current_context.span();
        let span_context = spanctx.span_context();

        assert_ne!(
            span_context.trace_id().to_string(),
            "00000000000000000000000000000000",
        );

        //global::shutdown_tracer_provider();
    }
}
