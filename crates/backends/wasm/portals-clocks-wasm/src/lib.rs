//! WASM implementation of pith-clocks.
//!
//! Uses browser APIs:
//! - `Date.now()` for wall clock
//! - `performance.now()` for monotonic clock
//! - `setTimeout` for timers

use rhizome_rhi_portals_clocks::{MonotonicClock, WallClock};
use std::time::Duration;

/// Wall clock using JavaScript Date.
#[derive(Debug, Default, Clone, Copy)]
pub struct JsWallClock;

impl WallClock for JsWallClock {
    fn now(&self) -> (u64, u32) {
        // Date.now() returns milliseconds since Unix epoch
        let millis = js_sys::Date::now() as u64;
        let secs = millis / 1000;
        let nanos = ((millis % 1000) * 1_000_000) as u32;
        (secs, nanos)
    }

    fn resolution(&self) -> (u64, u32) {
        // JavaScript Date has millisecond resolution
        (0, 1_000_000)
    }
}

/// Monotonic clock using Performance.now().
#[derive(Debug, Clone)]
pub struct PerformanceClock {
    /// The epoch value from performance.now() when this clock was created.
    epoch_ms: f64,
}

impl Default for PerformanceClock {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceClock {
    /// Create a new monotonic clock with epoch at creation time.
    pub fn new() -> Self {
        Self {
            epoch_ms: performance_now(),
        }
    }
}

impl MonotonicClock for PerformanceClock {
    fn now(&self) -> u64 {
        let elapsed_ms = performance_now() - self.epoch_ms;
        // Convert milliseconds to nanoseconds
        (elapsed_ms * 1_000_000.0) as u64
    }

    fn resolution(&self) -> u64 {
        // performance.now() has sub-millisecond resolution (typically microseconds)
        // but browsers may reduce precision for security (Spectre mitigations)
        1_000_000 // Report as 1ms to be conservative
    }

    fn subscribe_duration(&self, duration: Duration) -> impl std::future::Future<Output = ()> {
        let millis = duration.as_millis() as u32;
        gloo_timers::future::TimeoutFuture::new(millis)
    }

    fn subscribe_instant(&self, instant: u64) -> impl std::future::Future<Output = ()> {
        let now = self.now();
        let delay = if instant > now {
            Duration::from_nanos(instant - now)
        } else {
            Duration::ZERO
        };
        let millis = delay.as_millis() as u32;
        gloo_timers::future::TimeoutFuture::new(millis)
    }
}

/// Get the current performance.now() value.
fn performance_now() -> f64 {
    web_sys::window()
        .expect("should have window")
        .performance()
        .expect("should have performance")
        .now()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn wall_clock_returns_reasonable_time() {
        let clock = JsWallClock;
        let (secs, _nanos) = clock.now();
        // Should be after 2020 (1577836800) and before 2100 (4102444800)
        assert!(secs > 1577836800);
        assert!(secs < 4102444800);
    }

    #[wasm_bindgen_test]
    fn monotonic_clock_increases() {
        let clock = PerformanceClock::new();
        let t1 = clock.now();
        // Small busy loop to ensure time passes
        for _ in 0..10000 {
            let _ = js_sys::Date::now();
        }
        let t2 = clock.now();
        assert!(t2 >= t1);
    }

    #[wasm_bindgen_test]
    async fn subscribe_duration_works() {
        let clock = PerformanceClock::new();
        let start = clock.now();
        clock.subscribe_duration(Duration::from_millis(50)).await;
        let elapsed = clock.now() - start;
        // At least 40ms (allowing some tolerance)
        assert!(elapsed >= 40_000_000);
    }
}
