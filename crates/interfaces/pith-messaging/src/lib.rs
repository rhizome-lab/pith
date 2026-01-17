//! Message queue interfaces.
//!
//! Based on WASI messaging. Follows capability-based design: traits operate
//! on already-opened channels/topics. Backends provide constructors.
//!
//! See ADR-0004 for rationale.

use std::fmt;
use std::future::Future;
use std::time::Duration;

/// Messaging errors.
#[derive(Debug)]
pub enum Error {
    Closed,
    Timeout,
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Closed => write!(f, "channel closed"),
            Error::Timeout => write!(f, "timeout"),
            Error::Other(msg) => write!(f, "messaging error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// A message with payload and metadata.
#[derive(Debug, Clone)]
pub struct Message {
    /// Message payload.
    pub data: Vec<u8>,
    /// Optional metadata/headers.
    pub metadata: Vec<(String, String)>,
}

impl Message {
    /// Create a new message with data.
    pub fn new(data: impl Into<Vec<u8>>) -> Self {
        Self {
            data: data.into(),
            metadata: Vec::new(),
        }
    }

    /// Add metadata to the message.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }
}

/// A message sender.
///
/// This trait operates on an already-opened sender endpoint.
/// The sender is obtained from a backend constructor.
pub trait Sender {
    /// Send a message.
    fn send(&self, message: Message) -> impl Future<Output = Result<(), Error>>;
}

/// A message receiver.
///
/// This trait operates on an already-opened receiver endpoint.
/// The receiver is obtained from a backend constructor.
pub trait Receiver {
    /// Receive a message, waiting indefinitely.
    fn receive(&self) -> impl Future<Output = Result<Message, Error>>;

    /// Receive a message with timeout.
    fn receive_timeout(&self, timeout: Duration) -> impl Future<Output = Result<Message, Error>>;

    /// Try to receive a message without blocking.
    fn try_receive(&self) -> impl Future<Output = Result<Option<Message>, Error>>;
}

/// A channel for point-to-point messaging.
///
/// This trait operates on an already-opened channel.
/// The channel is obtained from a backend constructor.
pub trait Channel {
    /// The sender type.
    type Sender: Sender;
    /// The receiver type.
    type Receiver: Receiver;

    /// Create a new sender/receiver pair from this channel.
    fn create(&self) -> (Self::Sender, Self::Receiver);
}

/// A subscriber that receives messages from a topic.
pub trait Subscriber: Receiver {
    /// Unsubscribe from the topic.
    fn unsubscribe(self) -> impl Future<Output = Result<(), Error>>;
}

/// A topic for publish/subscribe messaging.
///
/// This trait operates on an already-opened topic.
/// The topic is obtained from a backend constructor.
///
/// ```ignore
/// // Backend provides construction
/// let topic = messaging_backend.open_topic("events")?;
///
/// // Interface defines operations
/// topic.publish(Message::new(b"hello")).await?;
/// let subscriber = topic.subscribe().await?;
/// ```
pub trait Topic {
    /// The subscriber type.
    type Subscriber: Subscriber;

    /// Publish a message to all subscribers.
    fn publish(&self, message: Message) -> impl Future<Output = Result<(), Error>>;

    /// Subscribe to receive messages.
    fn subscribe(&self) -> impl Future<Output = Result<Self::Subscriber, Error>>;
}
