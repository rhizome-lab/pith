//! Clock interfaces.
//!
//! Based on WASI clocks.

use std::time::Duration;

/// A wall clock - tells the current time.
pub trait WallClock {
    /// Returns the current time as seconds and nanoseconds since Unix epoch.
    fn now(&self) -> (u64, u32);

    /// Returns the resolution of the clock.
    fn resolution(&self) -> (u64, u32);
}

/// A monotonic clock - measures elapsed time.
pub trait MonotonicClock {
    /// Returns the current value of the clock in nanoseconds.
    fn now(&self) -> u64;

    /// Returns the resolution of the clock in nanoseconds.
    fn resolution(&self) -> u64;

    /// Subscribe to a timer that completes after the given duration.
    fn subscribe_duration(&self, duration: Duration) -> impl Future<Output = ()>;

    /// Subscribe to a timer that completes at the given instant.
    fn subscribe_instant(&self, instant: u64) -> impl Future<Output = ()>;
}

use std::future::Future;
