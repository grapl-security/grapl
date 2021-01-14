use std::time::Duration;

use futures_retry::{ErrorHandler, RetryPolicy};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use rusoto_core::RusotoError;

pub struct RetryHandler {
    max_attempts: u8,
    rng: XorShiftRng,
}

impl RetryHandler {
    pub fn new(attempts: u8) -> Self {
        Self {
            max_attempts: attempts,
            rng: XorShiftRng::from_seed([0; 16]), // Unseeded
        }
    }
}

impl<E> ErrorHandler<RusotoError<E>> for RetryHandler {
    type OutError = RusotoError<E>;

    fn handle(&mut self, attempt: usize, e: RusotoError<E>) -> RetryPolicy<Self::OutError> {
        if attempt == self.max_attempts as usize {
            return RetryPolicy::ForwardError(e);
        }

        match e {
            RusotoError::HttpDispatch(_) | RusotoError::Unknown(_) => {
                let jitter: u64 = self.rng.gen_range(10 .. 50);
                let backoff = jitter + (attempt as u64 * 10);
                RetryPolicy::WaitRetry(Duration::from_millis(backoff))
            }
            _ => RetryPolicy::ForwardError(e),
        }
    }
}
