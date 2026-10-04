use alloc::sync::Arc;
use core::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Debug, Default)]
pub struct Generation {
    counter: Arc<AtomicU64>,
}

impl Generation {
    pub fn advance(&self) -> u64 {
        let previous = self.counter.fetch_add(1, Ordering::SeqCst);
        let next = previous + 1;

        assert!(next > previous);

        next
    }

    pub fn is_current(&self, generation: u64) -> bool {
        debug_assert!(generation > 0);

        self.counter.load(Ordering::SeqCst) == generation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_superseded_generation_is_not_current() {
        let generation = Generation::default();
        let first = generation.advance();
        let second = generation.advance();

        assert!(!generation.is_current(first));
        assert!(generation.is_current(second));
    }

    #[test]
    fn clones_share_one_counter() {
        let generation = Generation::default();
        let observer = generation.clone();
        let issued = generation.advance();

        assert!(observer.is_current(issued));
    }

    #[test]
    fn generations_never_repeat() {
        let generation = Generation::default();
        let mut seen = Vec::new();

        for _ in 0..8 {
            let issued = generation.advance();

            assert!(!seen.contains(&issued));

            seen.push(issued);
        }

        assert!(generation.is_current(8));
    }
}
