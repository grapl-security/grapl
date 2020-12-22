use std::time::Duration;

use futures_retry::{ErrorHandler, RetryPolicy};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use rusoto_core::RusotoError;

pub struct RetryHandler {
    attempt: u8,
    max_attempts: u8,
    rng: XorShiftRng,
}

impl RetryHandler {
    pub fn new(attempts: u8) -> Self {
        Self {
            attempt: 0,
            max_attempts: attempts,
            rng: XorShiftRng::from_seed([0; 16]), // Unseeded
        }
    }
}

impl<E> ErrorHandler<RusotoError<E>> for RetryHandler {
    type OutError = RusotoError<E>;

    fn handle(&mut self, e: RusotoError<E>) -> RetryPolicy<RusotoError<E>> {
        if self.attempt == self.max_attempts {
            return RetryPolicy::ForwardError(e);
        }
        self.attempt += 1;
        match e {
            RusotoError::HttpDispatch(_) | RusotoError::Unknown(_) => {
                let jitter: u64 = self.rng.gen_range(10, 50);
                let backoff = jitter + (self.attempt as u64 * 10);
                RetryPolicy::WaitRetry(Duration::from_millis(backoff))
            }
            _ => RetryPolicy::ForwardError(e),
        }
    }

    fn ok(&mut self) {
        self.attempt = 0;
    }
}
