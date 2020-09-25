use crate::metrics::SysmonSubgraphGeneratorMetrics;
use crate::models::SysmonTryFrom;
use async_trait::async_trait;
use failure::bail;
use graph_descriptions::graph_description::*;
use grapl_observe::log_time;
use log::*;
use sqs_lambda::cache::{Cache, CacheResponse};
use sqs_lambda::event_handler::{Completion, EventHandler, OutputEvent};
use sysmon::Event;

#[derive(Clone)]
pub(crate) struct SysmonSubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    cache: C,
    metrics: SysmonSubgraphGeneratorMetrics,
}

impl<C> SysmonSubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    pub fn new(cache: C, metrics: SysmonSubgraphGeneratorMetrics) -> Self {
        Self { cache, metrics }
    }
}

#[async_trait]
impl<C> EventHandler for SysmonSubgraphGenerator<C>
where
    C: Cache + Clone + Send + Sync + 'static,
{
    type InputEvent = Vec<u8>;
    type OutputEvent = Graph;
    type Error = sqs_lambda::error::Error;

    async fn handle_event(
        &mut self,
        events: Vec<u8>,
    ) -> OutputEvent<Self::OutputEvent, Self::Error> {
        info!("Handling raw event");

        let mut failed: Option<failure::Error> = None;

        let events: Vec<_> = log_time!(
            "event split",
            events
                .split(|i| &[*i][..] == &b"\n"[..])
                .map(String::from_utf8_lossy)
                .filter(|event| {
                    (!event.is_empty() && event != "\n")
                        && (event.contains(&"EventID>1<"[..])
                            || event.contains(&"EventID>3<"[..])
                            || event.contains(&"EventID>11<"[..]))
                })
                .collect()
        );

        info!("Handling {} events", events.len());

        let mut identities = Vec::with_capacity(events.len());

        let mut final_subgraph = Graph::new(0);

        for event in events {
            let des_event = Event::from_str(&event);
            let event = match des_event {
                Ok(event) => event,
                Err(e) => {
                    warn!("Failed to deserialize event: {}, {}", e, event);
                    failed = Some(
                        (|| {
                            bail!("Failed: {}", e);
                            Ok(())
                        })()
                        .unwrap_err(),
                    );
                    continue;
                }
            };

            match self.cache.get(event.clone()).await {
                Ok(CacheResponse::Hit) => {
                    info!("Got cached response");
                    continue;
                }
                Err(e) => warn!("Cache failed with: {:?}", e),
                _ => (),
            };

            let graph = match Graph::try_from(event.clone()) {
                Ok(subgraph) => subgraph,
                Err(e) => {
                    failed = Some(e);
                    continue;
                }
            };

            identities.push(event);

            final_subgraph.merge(&graph);
        }

        info!("Completed mapping {} subgraphs", identities.len());
        self.metrics.report_handle_event_success(&failed);

        let mut completed = if let Some(ref e) = failed {
            OutputEvent::new(Completion::Partial((
                final_subgraph,
                sqs_lambda::error::Error::ProcessingError(e.to_string()),
            )))
        } else {
            OutputEvent::new(Completion::Total(final_subgraph))
        };

        identities
            .into_iter()
            .for_each(|identity| completed.add_identity(identity));

        completed
    }
}
