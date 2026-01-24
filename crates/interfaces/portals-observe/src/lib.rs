//! Observability/telemetry interfaces.
//!
//! Based on WASI observe.

/// A span for distributed tracing.
pub trait Span {
    /// Set an attribute on this span.
    fn set_attribute(&self, key: &str, value: &str);

    /// Add an event to this span.
    fn add_event(&self, name: &str);

    /// End the span.
    fn end(self);
}

/// A tracer that creates spans.
pub trait Tracer {
    /// The span type.
    type Span: Span;

    /// Start a new span.
    fn start_span(&self, name: &str) -> Self::Span;

    /// Start a span as a child of another span.
    fn start_span_with_parent(&self, name: &str, parent: &Self::Span) -> Self::Span;
}

/// A counter metric (monotonically increasing).
pub trait Counter {
    /// Add to the counter.
    fn add(&self, value: u64);
}

/// A gauge metric (can go up or down).
pub trait Gauge {
    /// Set the gauge value.
    fn set(&self, value: f64);
}

/// A histogram metric (records distributions).
pub trait Histogram {
    /// Record a value.
    fn record(&self, value: f64);
}

/// A metrics provider.
pub trait Metrics {
    /// The counter type.
    type Counter: Counter;
    /// The gauge type.
    type Gauge: Gauge;
    /// The histogram type.
    type Histogram: Histogram;

    /// Create or get a counter.
    fn counter(&self, name: &str, description: &str) -> Self::Counter;

    /// Create or get a gauge.
    fn gauge(&self, name: &str, description: &str) -> Self::Gauge;

    /// Create or get a histogram.
    fn histogram(&self, name: &str, description: &str) -> Self::Histogram;
}
