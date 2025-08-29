use std::sync::atomic::{AtomicUsize, Ordering};

static TERMINAL_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn next_terminal_number() -> usize {
    TERMINAL_COUNTER.fetch_add(1, Ordering::Relaxed)
}
