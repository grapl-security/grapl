use std::sync::{
    atomic::{
        AtomicBool,
        Ordering,
    },
    Arc,
};

/// ShortCircuit provides an API for signaling between threads (or tasks)
/// when they should stop working. For example, if a given query is processed
/// concurrently by N tasks, and one task finds a matching graph, it can communicate
/// to the other tasks that they should stop working.
#[derive(Clone)]
pub struct ShortCircuit {
    short_circuit: Arc<AtomicBool>,
}

impl Default for ShortCircuit {
    fn default() -> Self {
        Self::new()
    }
}

impl ShortCircuit {
    pub fn new() -> Self {
        Self {
            short_circuit: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get_short_circuit(&self) -> bool {
        self.short_circuit.as_ref().load(Ordering::Acquire)
    }

    pub fn set_short_circuit(&self) {
        self.short_circuit.as_ref().store(true, Ordering::Release)
    }
}
