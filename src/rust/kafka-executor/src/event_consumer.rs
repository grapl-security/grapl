use grapl_config::env_helpers::FromEnv;
use rdkafka::{
    config::{
        FromClientConfig,
        RDKafkaLogLevel,
    },
    consumer::{
        Consumer,
        ConsumerContext,
        DefaultConsumerContext,
        StreamConsumer,
    },
    util::{
        AsyncRuntime,
        DefaultRuntime,
    },
};

const KAFKA_CONSUMER_TOPIC: &str = "KAFKA_CONSUMER_TOPIC";
const KAFKA_CONSUMER_BROKERS: &str = "KAFKA_CONSUMER_BROKERS";
const KAFKA_CONSUMER_GROUP_ID: &str = "KAFKA_CONSUMER_GROUP_ID";
const KAFKA_CONSUMER_CLIENT_ID: &str = "KAFKA_CONSUMER_CLIENT_ID";
const KAFKA_CONSUMER_SESSION_TIMEOUT_MS: &str = "KAFKA_CONSUMER_SESSION_TIMEOUT_MS";
const KAFKA_CONSUMER_ENABLE_AUTO_COMMIT: &str = "KAFKA_CONSUMER_ENABLE_AUTO_COMMIT";
const KAFKA_CONSUMER_MAX_POLL_INTERVAL_MS: &str = "KAFKA_CONSUMER_MAX_POLL_INTERVAL_MS";
const KAFKA_CONSUMER_REQUEST_TIMEOUT_MS: &str = "KAFKA_CONSUMER_REQUEST_TIMEOUT_MS";

pub struct EventConsumer<C = DefaultConsumerContext, R = DefaultRuntime>
where
    C: ConsumerContext + 'static,
    R: AsyncRuntime,
{
    pub(crate) consumer: StreamConsumer<C, R>,
    pub(crate) topic_name: String,
}

// We may want to add more tunables https://github.com/edenhill/librdkafka/blob/master/INTRODUCTION.md#performance
impl<R> FromEnv<Self> for EventConsumer<DefaultConsumerContext, R>
where
    R: AsyncRuntime,
{
    fn from_env() -> Self {
        let topic = std::env::var(KAFKA_CONSUMER_TOPIC).expect(KAFKA_CONSUMER_TOPIC);
        let brokers = std::env::var(KAFKA_CONSUMER_BROKERS).expect(KAFKA_CONSUMER_BROKERS);
        let consumer_group_id =
            std::env::var(KAFKA_CONSUMER_GROUP_ID).expect(KAFKA_CONSUMER_GROUP_ID);
        let consumer_client_id =
            std::env::var(KAFKA_CONSUMER_CLIENT_ID).expect(KAFKA_CONSUMER_CLIENT_ID);

        let mut client_config = rdkafka::ClientConfig::new();
        client_config
            .set("group.id", consumer_group_id)
            .set("client.id", consumer_client_id)
            .set("bootstrap.servers", brokers);

        if let Ok(session_timeout_ms) = std::env::var(KAFKA_CONSUMER_SESSION_TIMEOUT_MS) {
            client_config.set("session.timeout.ms", session_timeout_ms);
        }

        if let Ok(enable_auto_commit) = std::env::var(KAFKA_CONSUMER_ENABLE_AUTO_COMMIT) {
            assert!(enable_auto_commit == "true" || enable_auto_commit == "false");
            client_config.set("enable.auto.commit", enable_auto_commit);
        }

        if let Ok(max_poll_interval_ms) = std::env::var(KAFKA_CONSUMER_MAX_POLL_INTERVAL_MS) {
            client_config.set("max.poll.interval.ms", max_poll_interval_ms);
        }

        if let Ok(request_timeout_ms) = std::env::var(KAFKA_CONSUMER_REQUEST_TIMEOUT_MS) {
            client_config.set("request.timeout.ms", request_timeout_ms);
        }

        client_config.set_log_level(RDKafkaLogLevel::Debug);

        let consumer =
            StreamConsumer::from_config(&client_config).expect("StreamConsumer::from_config");
        consumer
            .subscribe(&[&topic])
            .expect("Can't subscribe to specified topic");

        Self {
            consumer,
            topic_name: topic,
        }
    }
}
