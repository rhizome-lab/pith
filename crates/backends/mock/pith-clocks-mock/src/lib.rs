//! Mock implementation of pith-clocks for testing.
//!
//! Provides controllable clocks that allow tests to manipulate time.

use rhizome_pith_clocks::{MonotonicClock, WallClock};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// A wall clock with controllable time.
#[derive(Debug, Clone)]
pub struct MockWallClock {
    secs: Arc<AtomicU64>,
    nanos: Arc<AtomicU64>,
}

impl Default for MockWallClock {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl MockWallClock {
    /// Create a new mock wall clock starting at the given time.
    pub fn new(secs: u64, nanos: u32) -> Self {
        Self {
            secs: Arc::new(AtomicU64::new(secs)),
            nanos: Arc::new(AtomicU64::new(nanos as u64)),
        }
    }

    /// Create a mock wall clock at Unix epoch (1970-01-01 00:00:00 UTC).
    pub fn at_epoch() -> Self {
        Self::new(0, 0)
    }

    /// Set the current time.
    pub fn set(&self, secs: u64, nanos: u32) {
        self.secs.store(secs, Ordering::SeqCst);
        self.nanos.store(nanos as u64, Ordering::SeqCst);
    }

    /// Advance time by the given duration.
    pub fn advance(&self, duration: Duration) {
        let add_secs = duration.as_secs();
        let add_nanos = duration.subsec_nanos() as u64;

        let old_nanos = self.nanos.load(Ordering::SeqCst);
        let total_nanos = old_nanos + add_nanos;

        let extra_secs = total_nanos / 1_000_000_000;
        let new_nanos = total_nanos % 1_000_000_000;

        self.secs.fetch_add(add_secs + extra_secs, Ordering::SeqCst);
        self.nanos.store(new_nanos, Ordering::SeqCst);
    }
}

impl WallClock for MockWallClock {
    fn now(&self) -> (u64, u32) {
        (
            self.secs.load(Ordering::SeqCst),
            self.nanos.load(Ordering::SeqCst) as u32,
        )
    }

    fn resolution(&self) -> (u64, u32) {
        (0, 1)
    }
}

/// A monotonic clock with controllable time.
#[derive(Debug, Clone)]
pub struct MockMonotonicClock {
    nanos: Arc<AtomicU64>,
}

impl Default for MockMonotonicClock {
    fn default() -> Self {
        Self::new()
    }
}

impl MockMonotonicClock {
    /// Create a new mock monotonic clock starting at 0.
    pub fn new() -> Self {
        Self {
            nanos: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Create a mock monotonic clock starting at the given nanosecond value.
    pub fn at(nanos: u64) -> Self {
        Self {
            nanos: Arc::new(AtomicU64::new(nanos)),
        }
    }

    /// Set the current time in nanoseconds.
    pub fn set(&self, nanos: u64) {
        self.nanos.store(nanos, Ordering::SeqCst);
    }

    /// Advance time by the given duration.
    pub fn advance(&self, duration: Duration) {
        self.nanos
            .fetch_add(duration.as_nanos() as u64, Ordering::SeqCst);
    }

    /// Advance time by the given number of nanoseconds.
    pub fn advance_nanos(&self, nanos: u64) {
        self.nanos.fetch_add(nanos, Ordering::SeqCst);
    }
}

impl MonotonicClock for MockMonotonicClock {
    fn now(&self) -> u64 {
        self.nanos.load(Ordering::SeqCst)
    }

    fn resolution(&self) -> u64 {
        1
    }

    fn subscribe_duration(&self, _duration: Duration) -> impl std::future::Future<Output = ()> {
        // In mock mode, timers complete immediately.
        // Tests should advance time and poll futures manually if needed.
        std::future::ready(())
    }

    fn subscribe_instant(&self, _instant: u64) -> impl std::future::Future<Output = ()> {
        std::future::ready(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_clock_set_and_get() {
        let clock = MockWallClock::new(1000, 500);
        assert_eq!(clock.now(), (1000, 500));

        clock.set(2000, 0);
        assert_eq!(clock.now(), (2000, 0));
    }

    #[test]
    fn wall_clock_advance() {
        let clock = MockWallClock::new(100, 500_000_000);
        clock.advance(Duration::from_millis(600));
        assert_eq!(clock.now(), (101, 100_000_000));
    }

    #[test]
    fn monotonic_clock_advance() {
        let clock = MockMonotonicClock::new();
        assert_eq!(clock.now(), 0);

        clock.advance(Duration::from_secs(1));
        assert_eq!(clock.now(), 1_000_000_000);

        clock.advance_nanos(500);
        assert_eq!(clock.now(), 1_000_000_500);
    }

    #[test]
    fn clocks_are_clone() {
        let clock = MockMonotonicClock::new();
        let clone = clock.clone();

        clock.advance(Duration::from_secs(1));
        // Clone shares the same underlying state
        assert_eq!(clone.now(), 1_000_000_000);
    }
}
