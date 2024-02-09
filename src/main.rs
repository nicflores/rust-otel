mod honeycomb_tracer;
mod jaeger_tracer;

use std::env;

use opentelemetry::trace::TraceContextExt;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace as sdktrace;
use tonic::metadata::MetadataMap;
use tracing::{span, Level};
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[tokio::main]
async fn main() {
    let honeycomb_key = env::var("HONEYCOMB_KEY").unwrap();
    let mut oltp_meta = MetadataMap::new();
    oltp_meta.insert("x-honeycomb-team", honeycomb_key.parse().unwrap());
    oltp_meta.insert("x-honeycomb-dataset", "testdataset".parse().unwrap());

    let config: sdktrace::Config =
        opentelemetry_sdk::trace::config().with_resource(opentelemetry_sdk::Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", "testdataset"),
        ]));

    let exporter: opentelemetry_otlp::TonicExporterBuilder = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("https://api.honeycomb.io:443")
        .with_metadata(oltp_meta);

    honeycomb_tracer::honeycomb_tracer(config, exporter);

    let span = span!(Level::INFO, "read_message_from_sqs");
    let _enter = span.enter();

    let current_span = tracing::Span::current();
    let current_context = current_span.context();
    let spanctx = current_context.span();
    let span_context = spanctx.span_context();

    let trace_id = span_context.trace_id();
    let span_id = span_context.span_id();

    println!("Trace ID: {:#?}, Span ID: {:#?}", trace_id, span_id);
}
