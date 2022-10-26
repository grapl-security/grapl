#![cfg(feature = "integration_tests")]

use std::time::Duration;

use grapl_metrics::setup_metrics::setup_metrics;
use opentelemetry::{
    global,
    Context,
    Key,
    KeyValue,
};

const TEST_KEY: Key = Key::from_static_str("test_key");

/// Unfortunately, we don't really have a way to automatically test that these
/// metrics made it all the way to Lightstep.
/// -- However! --
/// You can manually confirm they showed up in the `otel-collector` by doing:
/// 1. add `logging` to the metrics exporters in pulumi/infra/observability_env_vars.py
/// 2. grep -A 5 metrics-integ-test $YOUR_TEST_ARTIFACTS/otel-collector/*
#[test_log::test(tokio::test(flavor = "multi_thread"))]
async fn test_metrics_primitives_histogram_gauge_counter() -> eyre::Result<()> {
    let common_attrs: [KeyValue; 4] = [
        TEST_KEY.i64(10),
        KeyValue::new("A", "1"),
        KeyValue::new("B", "2"),
        KeyValue::new("C", "3"),
    ];

    let _metrics = setup_metrics()?;
    let cx = Context::current();

    let meter = global::meter("metrics-integ-test");

    let histogram = meter.f64_histogram("test-histogram").init();
    histogram.record(&cx, 5.5, &(common_attrs.clone()));

    let counter = meter.i64_up_down_counter("test-counter").with_description("counter with total of 3").init();
    counter.add(&cx, 1, common_attrs.as_ref());
    counter.add(&cx, 2, common_attrs.as_ref());

    let gauge = meter
        .f64_observable_gauge("test-gauge")
        .with_description("A gauge set to 1.0")
        .init();
    // You can only call `.observe` inside register_callback(...).
    meter.register_callback(move |cx| gauge.observe(cx, 1.0, common_attrs.clone().as_ref()))?;

    // wait a sec for the metrics to flush
    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(())
}
