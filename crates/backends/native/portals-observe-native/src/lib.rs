//! Native observability implementation.
//!
//! Provides no-op implementations for when telemetry is not needed,
//! plus simple in-memory implementations for testing.

use portals_observe::{Counter, Gauge, Histogram, Metrics, Span, Tracer};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// A no-op span that does nothing.
#[derive(Debug, Default)]
pub struct NoopSpan;

impl Span for NoopSpan {
    fn set_attribute(&self, _key: &str, _value: &str) {}
    fn add_event(&self, _name: &str) {}
    fn end(self) {}
}

/// A no-op tracer that produces no-op spans.
#[derive(Debug, Default)]
pub struct NoopTracer;

impl NoopTracer {
    /// Create a new no-op tracer.
    pub fn new() -> Self {
        Self
    }
}

impl Tracer for NoopTracer {
    type Span = NoopSpan;

    fn start_span(&self, _name: &str) -> Self::Span {
        NoopSpan
    }

    fn start_span_with_parent(&self, _name: &str, _parent: &Self::Span) -> Self::Span {
        NoopSpan
    }
}

/// A no-op counter.
#[derive(Debug, Default)]
pub struct NoopCounter;

impl Counter for NoopCounter {
    fn add(&self, _value: u64) {}
}

/// A no-op gauge.
#[derive(Debug, Default)]
pub struct NoopGauge;

impl Gauge for NoopGauge {
    fn set(&self, _value: f64) {}
}

/// A no-op histogram.
#[derive(Debug, Default)]
pub struct NoopHistogram;

impl Histogram for NoopHistogram {
    fn record(&self, _value: f64) {}
}

/// No-op metrics provider.
#[derive(Debug, Default)]
pub struct NoopMetrics;

impl NoopMetrics {
    /// Create a new no-op metrics provider.
    pub fn new() -> Self {
        Self
    }
}

impl Metrics for NoopMetrics {
    type Counter = NoopCounter;
    type Gauge = NoopGauge;
    type Histogram = NoopHistogram;

    fn counter(&self, _name: &str, _description: &str) -> Self::Counter {
        NoopCounter
    }

    fn gauge(&self, _name: &str, _description: &str) -> Self::Gauge {
        NoopGauge
    }

    fn histogram(&self, _name: &str, _description: &str) -> Self::Histogram {
        NoopHistogram
    }
}

/// An in-memory counter for testing.
#[derive(Debug, Default)]
pub struct MemoryCounter {
    value: AtomicU64,
}

impl MemoryCounter {
    /// Get the current value.
    pub fn value(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

impl Counter for MemoryCounter {
    fn add(&self, value: u64) {
        self.value.fetch_add(value, Ordering::Relaxed);
    }
}

/// An in-memory gauge for testing.
#[derive(Debug, Default)]
pub struct MemoryGauge {
    value: RwLock<f64>,
}

impl MemoryGauge {
    /// Get the current value.
    pub fn value(&self) -> f64 {
        *self.value.read().unwrap()
    }
}

impl Gauge for MemoryGauge {
    fn set(&self, value: f64) {
        *self.value.write().unwrap() = value;
    }
}

/// An in-memory histogram for testing.
#[derive(Debug, Default)]
pub struct MemoryHistogram {
    values: RwLock<Vec<f64>>,
}

impl MemoryHistogram {
    /// Get all recorded values.
    pub fn values(&self) -> Vec<f64> {
        self.values.read().unwrap().clone()
    }

    /// Get the count of recorded values.
    pub fn count(&self) -> usize {
        self.values.read().unwrap().len()
    }
}

impl Histogram for MemoryHistogram {
    fn record(&self, value: f64) {
        self.values.write().unwrap().push(value);
    }
}

/// Shared counter wrapper.
#[derive(Debug, Clone, Default)]
pub struct SharedCounter(Arc<MemoryCounter>);

impl SharedCounter {
    /// Get the current value.
    pub fn value(&self) -> u64 {
        self.0.value()
    }
}

impl Counter for SharedCounter {
    fn add(&self, value: u64) {
        self.0.add(value);
    }
}

/// Shared gauge wrapper.
#[derive(Debug, Clone, Default)]
pub struct SharedGauge(Arc<MemoryGauge>);

impl SharedGauge {
    /// Get the current value.
    pub fn value(&self) -> f64 {
        self.0.value()
    }
}

impl Gauge for SharedGauge {
    fn set(&self, value: f64) {
        self.0.set(value);
    }
}

/// Shared histogram wrapper.
#[derive(Debug, Clone, Default)]
pub struct SharedHistogram(Arc<MemoryHistogram>);

impl SharedHistogram {
    /// Get all recorded values.
    pub fn values(&self) -> Vec<f64> {
        self.0.values()
    }

    /// Get the count of recorded values.
    pub fn count(&self) -> usize {
        self.0.count()
    }
}

impl Histogram for SharedHistogram {
    fn record(&self, value: f64) {
        self.0.record(value);
    }
}

/// In-memory metrics provider for testing.
#[derive(Debug, Default)]
pub struct MemoryMetrics;

impl MemoryMetrics {
    /// Create a new in-memory metrics provider.
    pub fn new() -> Self {
        Self
    }
}

impl Metrics for MemoryMetrics {
    type Counter = SharedCounter;
    type Gauge = SharedGauge;
    type Histogram = SharedHistogram;

    fn counter(&self, _name: &str, _description: &str) -> Self::Counter {
        SharedCounter(Arc::new(MemoryCounter::default()))
    }

    fn gauge(&self, _name: &str, _description: &str) -> Self::Gauge {
        SharedGauge(Arc::new(MemoryGauge::default()))
    }

    fn histogram(&self, _name: &str, _description: &str) -> Self::Histogram {
        SharedHistogram(Arc::new(MemoryHistogram::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_tracer() {
        let tracer = NoopTracer::new();
        let span = tracer.start_span("test");
        span.set_attribute("key", "value");
        span.add_event("event");
        span.end();
    }

    #[test]
    fn noop_metrics() {
        let metrics = NoopMetrics::new();
        let counter = metrics.counter("requests", "Total requests");
        counter.add(1);
        let gauge = metrics.gauge("temp", "Temperature");
        gauge.set(42.0);
        let histogram = metrics.histogram("latency", "Request latency");
        histogram.record(0.5);
    }

    #[test]
    fn memory_counter() {
        let counter = MemoryCounter::default();
        assert_eq!(counter.value(), 0);
        counter.add(5);
        counter.add(3);
        assert_eq!(counter.value(), 8);
    }

    #[test]
    fn memory_gauge() {
        let gauge = MemoryGauge::default();
        gauge.set(10.0);
        assert_eq!(gauge.value(), 10.0);
        gauge.set(20.0);
        assert_eq!(gauge.value(), 20.0);
    }

    #[test]
    fn memory_histogram() {
        let histogram = MemoryHistogram::default();
        histogram.record(1.0);
        histogram.record(2.0);
        histogram.record(3.0);
        assert_eq!(histogram.count(), 3);
        assert_eq!(histogram.values(), vec![1.0, 2.0, 3.0]);
    }
}
