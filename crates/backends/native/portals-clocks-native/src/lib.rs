//! Native implementation of pith-clocks.

use rhizome_rhi_portals_clocks::{MonotonicClock, WallClock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Wall clock using system time.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl WallClock for SystemClock {
    fn now(&self) -> (u64, u32) {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch");
        (duration.as_secs(), duration.subsec_nanos())
    }

    fn resolution(&self) -> (u64, u32) {
        // Most systems have nanosecond resolution, but actual precision varies
        (0, 1)
    }
}

/// Monotonic clock using std::time::Instant.
#[derive(Debug, Clone)]
pub struct StdMonotonicClock {
    epoch: Instant,
}

impl Default for StdMonotonicClock {
    fn default() -> Self {
        Self::new()
    }
}

impl StdMonotonicClock {
    /// Create a new monotonic clock with epoch at creation time.
    pub fn new() -> Self {
        Self {
            epoch: Instant::now(),
        }
    }
}

impl MonotonicClock for StdMonotonicClock {
    fn now(&self) -> u64 {
        self.epoch.elapsed().as_nanos() as u64
    }

    fn resolution(&self) -> u64 {
        1 // nanosecond
    }

    #[cfg(feature = "tokio")]
    fn subscribe_duration(&self, duration: Duration) -> impl std::future::Future<Output = ()> {
        tokio::time::sleep(duration)
    }

    #[cfg(not(feature = "tokio"))]
    fn subscribe_duration(&self, _duration: Duration) -> impl std::future::Future<Output = ()> {
        // Without a runtime, return a future that never completes
        std::future::pending()
    }

    #[cfg(feature = "tokio")]
    fn subscribe_instant(&self, instant: u64) -> impl std::future::Future<Output = ()> {
        let now = self.now();
        let delay = if instant > now {
            Duration::from_nanos(instant - now)
        } else {
            Duration::ZERO
        };
        tokio::time::sleep(delay)
    }

    #[cfg(not(feature = "tokio"))]
    fn subscribe_instant(&self, _instant: u64) -> impl std::future::Future<Output = ()> {
        std::future::pending()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_clock_returns_reasonable_time() {
        let clock = SystemClock;
        let (secs, _nanos) = clock.now();
        // Should be after 2020 (1577836800) and before 2100 (4102444800)
        assert!(secs > 1577836800);
        assert!(secs < 4102444800);
    }

    #[test]
    fn monotonic_clock_increases() {
        let clock = StdMonotonicClock::new();
        let t1 = clock.now();
        std::thread::sleep(Duration::from_millis(10));
        let t2 = clock.now();
        assert!(t2 > t1);
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn subscribe_duration_works() {
        let clock = StdMonotonicClock::new();
        let start = clock.now();
        clock.subscribe_duration(Duration::from_millis(50)).await;
        let elapsed = clock.now() - start;
        assert!(elapsed >= 50_000_000); // at least 50ms in nanos
    }
}
