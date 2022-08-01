use std::sync::{
    atomic::{
        AtomicBool,
        Ordering,
    },
    Arc,
};

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
