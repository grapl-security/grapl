# sqs-lambda
A Rust crate for writing AWS Lambdas that are triggered by SQS

This library allows you to turn single-event SQS-triggered lambdas into something that's
more like a streaming processor. This is useful for high throughput worklodas.

This example shows using this library alongside the lambda_runtime.

In this example we are:
* Triggered by SqsEvents from S3 Notify Events
* The S3PayloadRetriever uses the ZstdDecoder to download and decompress the payload
* We spawn 40 EventProcessors, wrapping our CustomService
* The completion handler will merge completed events before uploading to S3

```rust
fn my_handler(event: SqsEvent, ctx: Context) -> Result<(), HandlerError> {
    tokio_compat::run_std(
        async {

            let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL");
            info!("Queue Url: {}", queue_url);
            let output_bucket = "event-destination-bucket";

            let region = {
                let region_str = std::env::var("AWS_REGION").expect("AWS_REGION");
                Region::from_str(&region_str).expect("Region error")
            };

            info!("Defining consume policy");
            let consume_policy = ConsumePolicy::new(
                ctx, // Use the Context.deadline from the lambda_runtime
                Duration::from_secs(2), // Stop consuming when there's 2 seconds left in the runtime
                3, // If we get 3 empty receives in a row, stop consuming
            );

            info!("Defining consume policy");
            let (tx, shutdown_notify) = tokio::sync::oneshot::channel();

            info!("SqsConsumer");
            let sqs_consumer = SqsConsumerActor::new(
                SqsConsumer::new(SqsClient::new(region.clone()), queue_url.clone(), consume_policy, tx)
            );

            info!("SqsCompletionHandler");
            let sqs_completion_handler = SqsCompletionHandlerActor::new(
                SqsCompletionHandler::new(
                    SqsClient::new(region.clone()),
                    queue_url.to_string(),
                    SubgraphSerializer { proto: Vec::with_capacity(1024) },
                    S3EventEmitter::new(
                        S3Client::new(region.clone()),
                        bucket.to_owned(),
                        time_based_key_fn,
                    ),
                    CompletionPolicy::new(
                        1000, // Buffer up to 1000 messages
                        Duration::from_secs(30), // Buffer for up to 30 seconds
                    ),
                )
            );

            info!("EventProcessors");
            let event_processors: Vec<_> = (0..40)
                .map(|_| {
                    EventProcessorActor::new(EventProcessor::new(
                        sqs_consumer.clone(),
                        sqs_completion_handler.clone(),
                        CustomService {},
                        S3EventRetriever::new(S3Client::new(region.clone()), ZstdDecoder::default()),
                    ))
                })
                .collect();

            info!("Start Processing");

            futures::future::join_all(event_processors.iter().map(|ep| ep.start_processing())).await;

            let mut proc_iter = event_processors.iter().cycle();
            for event in event.records {
                let next_proc = proc_iter.next().unwrap();
                next_proc.process_event(
                    map_sqs_message(event)
                ).await;
            }

            info!("Waiting for shutdown notification");
            // Wait for the consumers to shutdown
            let _ = shutdown_notify.await;

            tokio::time::delay_for(Duration::from_millis(100)).await;
            info!("Consumer shutdown");

        });


    info!("Completed execution");
    Ok(())
}

```