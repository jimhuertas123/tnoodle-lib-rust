use std::sync::mpsc;
use std::thread;

use rand::rngs::ThreadRng;

use crate::puzzle::Puzzle;

/// Pre-generates scrambles in a background thread and caches them
/// for instant retrieval.
///
/// Port of Java's `ScrambleCacher`. Uses a bounded channel instead of
/// Java's synchronized circular buffer with wait/notify.
pub struct ScrambleCacher {
    receiver: mpsc::Receiver<String>,
    /// Sending a message on this channel tells the background thread to stop.
    _stop_tx: mpsc::Sender<()>,
}

impl ScrambleCacher {
    /// Create a cacher that pre-generates scrambles for the given puzzle.
    ///
    /// `generate_fn` is a closure that produces one scramble string.
    /// We take a closure rather than a `&dyn Puzzle` so the caller controls
    /// how the puzzle is accessed (avoids lifetime complexity with threads).
    pub fn new<F>(cache_size: usize, generate_fn: F) -> Self
    where
        F: Fn(&mut ThreadRng) -> String + Send + 'static,
    {
        let (tx, receiver) = mpsc::sync_channel(cache_size);
        let (stop_tx, stop_rx) = mpsc::channel();

        thread::Builder::new()
            .name("scramble-cacher".into())
            .spawn(move || {
                let mut rng = rand::rng();
                loop {
                    // Check if we've been asked to stop
                    if stop_rx.try_recv().is_ok() {
                        return;
                    }
                    let scramble = generate_fn(&mut rng);
                    // send blocks if channel is full (backpressure)
                    if tx.send(scramble).is_err() {
                        // Receiver dropped — stop
                        return;
                    }
                }
            })
            .expect("failed to spawn scramble cacher thread");

        Self {
            receiver,
            _stop_tx: stop_tx,
        }
    }

    /// Convenience constructor from a Puzzle impl.
    /// The puzzle must be `'static` since it moves to the background thread.
    pub fn for_puzzle<P>(puzzle: P, cache_size: usize) -> Self
    where
        P: Puzzle + 'static,
    {
        Self::new(cache_size, move |rng: &mut ThreadRng| {
            puzzle.generate_wca_scramble(rng)
        })
    }

    /// Get a pre-generated scramble. Blocks if the cache is empty.
    pub fn next_scramble(&self) -> String {
        self.receiver.recv().expect("scramble cacher thread panicked")
    }

    /// Try to get a scramble without blocking.
    pub fn try_next_scramble(&self) -> Option<String> {
        self.receiver.try_recv().ok()
    }
}

/// Default cache size matching the Java implementation.
pub const DEFAULT_CACHE_SIZE: usize = 100;
