use opentelemetry_otlp::TonicExporterBuilder;
use opentelemetry_sdk::trace as sdktrace;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

//#[allow(dead_code)]
pub fn jaeger_tracer() {
    let exporter = opentelemetry_jaeger::new_agent_pipeline()
        .install_simple()
        .expect("Failed to create Jaeger exporter");

    let telemetry = tracing_opentelemetry::layer().with_tracer(exporter);

    let subscriber = Registry::default().with(telemetry);
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");
}

#[cfg(test)]
mod tests {
    use super::jaeger_tracer;
    use opentelemetry::global;
    use opentelemetry::trace::TraceContextExt;
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::trace as sdktrace;
    use tonic::metadata::MetadataMap;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    #[test]
    fn jaeger_tracer_generates_nonzero_trace_id() {
        jaeger_tracer();
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
    }
}
