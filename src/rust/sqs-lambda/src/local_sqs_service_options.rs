use std::time::Duration;

use chrono::Utc;

use crate::{sqs_completion_handler::CompletionPolicy,
            sqs_consumer::ConsumePolicy};

#[derive(Default)]
pub struct LocalSqsServiceOptionsBuilder {
    completion_policy: Option<CompletionPolicy>,
    consume_policy: Option<ConsumePolicy>,
}
impl LocalSqsServiceOptionsBuilder {
    pub fn with_completion_policy(&mut self, arg: CompletionPolicy) -> &Self {
        self.completion_policy = Some(arg);
        self
    }

    pub fn with_minimal_buffer_completion_policy(&mut self) -> &Self {
        self.with_completion_policy(CompletionPolicy::new(
            1,                      // Buffer up to 1 message
            Duration::from_secs(1), // Buffer for up to 1 second
        ))
    }

    pub fn with_consume_policy(&mut self, arg: ConsumePolicy) -> &Self {
        self.consume_policy = Some(arg);
        self
    }

    pub fn build(self) -> LocalSqsServiceOptions {
        LocalSqsServiceOptions {
            completion_policy: self
                .completion_policy
                .unwrap_or_else(|| CompletionPolicy::new(10, Duration::from_secs(3))),
            consume_policy: self.consume_policy.unwrap_or_else(|| {
                ConsumePolicy::new(
                    Utc::now().timestamp_millis() + 10_000,
                    Duration::from_secs(5),
                    300,
                )
            }),
        }
    }
}

pub struct LocalSqsServiceOptions {
    pub completion_policy: CompletionPolicy,
    pub consume_policy: ConsumePolicy,
}
