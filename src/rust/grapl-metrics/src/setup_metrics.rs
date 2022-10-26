use std::panic::catch_unwind;

use opentelemetry::{
    global::shutdown_tracer_provider,
    metrics::{self,},
    runtime,
    sdk::{
        export::metrics::aggregation::cumulative_temporality_selector,
        metrics::{
            controllers::BasicController,
            selectors,
        },
    },
    Context,
};
use opentelemetry_otlp::{
    ExportConfig,
    WithExportConfig,
};
use tokio::task;
pub struct GraplMetrics {
    provider: BasicController,
    cx: Context,
}

impl Drop for GraplMetrics {
    fn drop(&mut self) {
        // invokes shutdown on all otel span processors
        shutdown_tracer_provider();
        self.provider
            .stop(&self.cx)
            .expect("Expected metrics provider stop");
    }
}

fn ensure_multithreaded_tokio_runtime() -> impl runtime::Runtime {
    // https://github.com/open-telemetry/opentelemetry-rust/blob/482772f2317e242e6b98bfe04ed42e4dac8cf77b/opentelemetry/src/sdk/trace/span_processor.rs#L182
    // https://github.com/open-telemetry/opentelemetry-rust/issues/786#issuecomment-1107884339

    // block_in_place() is currently the best heuristic for detecting if the
    // current runtime is multithreaded or not (it panics if single-thread).
    // tokio::main is multi by default, tokio::test is single by default.
    // Ultimately, this function is here primarily to help guide developers
    // who accidentally call `setup_metrics()` from a default `tokio::test`.

    // This will be better if https://github.com/tokio-rs/tokio/issues/5088
    // ever gets landed.
    let err_if_singlethread = catch_unwind(|| task::block_in_place(|| {}));
    match err_if_singlethread {
        Ok(_) => runtime::Tokio,
        Err(_) => panic!(
            r#"
            `setup_metrics()` called from a single-threaded Tokio runtime;
            you likely want `tokio::test(flavor = "multi_thread")`
        "#
        ),
    }
}

// Heavily based on
// https://github.com/open-telemetry/opentelemetry-rust/blob/main/examples/basic-otlp/src/main.rs
pub fn setup_metrics() -> metrics::Result<GraplMetrics> {
    let endpoint = std::env::var("OTEL_OTLP_METRICS_EXPORTER_ENDPOINT")
        .expect("OTEL_OTLP_METRICS_EXPORTER_ENDPOINT");

    let export_config = ExportConfig {
        endpoint,
        ..ExportConfig::default()
    };
    let runtime = ensure_multithreaded_tokio_runtime();
    let provider = opentelemetry_otlp::new_pipeline()
        .metrics(
            selectors::simple::inexpensive(),
            cumulative_temporality_selector(),
            runtime.clone(),
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_export_config(export_config),
        )
        .build()?;

    let cx = Context::current();
    provider.start(&cx, runtime)?;

    opentelemetry::global::set_meter_provider(provider.clone());
    Ok(GraplMetrics { provider, cx })
}
