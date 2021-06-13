#![allow(warnings)]

use std::collections::HashMap;

use grapl_config::init_grapl_env;
use rdkafka::config::FromClientConfig;

use crate::prometheus_parser::Value;

mod prometheus_parser;

fn producer_init() -> Result<rdkafka::producer::FutureProducer, Box<dyn std::error::Error>> {
    let brokers = "kafka:9092";  // todo: inject this
    let mut client_config = rdkafka::ClientConfig::new();
    client_config
        .set("client.id", "metrics-producer")
        .set("queue.buffering.max.ms", "0")
        .set("bootstrap.servers", brokers);
    tracing::info!(config=?client_config, message="Created Producer ClientConfig");

    let producer = rdkafka::producer::FutureProducer::from_config(&client_config)?;
    Ok(producer)
}

fn additional_tags() -> Vec<(String, String)> {
    let mut additional_tags = vec![];
    match std::env::var("ADDITIONAL_TAGS") {
        Ok(tags) => {
            for tag_pair in tags.split(",") {
                match tag_pair.split_once(":") {
                    Some((tag_a, tag_b)) => additional_tags.push((tag_a.to_string(), tag_b.to_string())),
                    None => {
                        tracing::error!(
                            message="Invalid tag pair",
                            tag_pair=tag_pair,
                        );
                    }
                }
            }
        },
        Err(_) => ()
    };
    additional_tags
}

fn get_timeout_ms() -> u32 {
    const DEFAULT: u32 = 1_000;
    let timeout_ms = std::env::var("METRIC_COLLECTION_INTERVAL");
    match timeout_ms {
        Ok(timeout_ms) => {
            timeout_ms.parse().unwrap_or_else(|e| {
                tracing::warn!(message="Invalid timeout_ms", error=?e, timeout_ms=?timeout_ms);
                DEFAULT
            })
        }
        Err(e) => {
            tracing::warn!(message="Missing or invalid timeout_ms", error=?e);
            DEFAULT
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (env, _guard) = grapl_config::init_grapl_env!();
    // kafka_metrics_exporter::KafkaMetricExporterBuilder::new(
    //     "grapl-metrics-topic",
    //     producer_init()?,
    // )
    //     .install();

    let mut scrape = prometheus_parser::Scrape::default();

    // todo: What other tags do we want here? Some sort of alpha identifier?
    loop {
        let mut tags = additional_tags();
        let metrics_response = reqwest::get("http://localhost:8080/debug/prometheus_metrics")
            .await?
            .text()
            .await?;

        let lines = metrics_response.lines();

        let metrics = scrape.parse(lines.into_iter())?;
        // // println!("{:?}", &metrics.docs);
        for sample in scrape.samples.drain(..) {
            dbg!(&sample);
            match sample.value {
                prometheus_parser::Value::Counter(m) => {
                    metrics::counter!(sample.metric, m as u64, &tags);
                }

                prometheus_parser::Value::Gauge(m) => {
                    metrics::gauge!(sample.metric, m, &tags);
                }
                prometheus_parser::Value::Histogram(m) => {
                    for hist_count in m {

                        let count = hist_count.count;
                        tags.push(("le".to_string(), hist_count.less_than.to_string()));
                        metrics::histogram!(sample.metric.clone(), count, &tags);
                        tags.pop();
                    }
                }

                prometheus_parser::Value::Summary(m) => {
                    for summary_count in m {
                        let count = summary_count.count;
                        tags.push(("quantile".to_string(), summary_count.quantile.to_string()));
                        metrics::histogram!(sample.metric.clone(), count, &tags);
                        tags.pop();
                    }
                }
                prometheus_parser::Value::Untyped(_) => {}
            }
        }

        let timeout_ms = get_timeout_ms();
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
