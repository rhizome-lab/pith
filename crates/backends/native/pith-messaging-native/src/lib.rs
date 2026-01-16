//! Native message queue implementation using tokio channels.

use rhizome_pith_messaging::{Channel, Error, Message, Messaging, Receiver, Sender, Subscriber, Topic};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};

/// A tokio mpsc sender.
pub struct MpscSender {
    tx: mpsc::Sender<Message>,
}

impl Sender for MpscSender {
    async fn send(&self, message: Message) -> Result<(), Error> {
        self.tx.send(message).await.map_err(|_| Error::Closed)
    }
}

/// A tokio mpsc receiver.
pub struct MpscReceiver {
    rx: tokio::sync::Mutex<mpsc::Receiver<Message>>,
}

impl Receiver for MpscReceiver {
    async fn receive(&self) -> Result<Message, Error> {
        self.rx
            .lock()
            .await
            .recv()
            .await
            .ok_or(Error::Closed)
    }

    async fn receive_timeout(&self, timeout: Duration) -> Result<Message, Error> {
        tokio::time::timeout(timeout, self.receive())
            .await
            .map_err(|_| Error::Timeout)?
    }

    async fn try_receive(&self) -> Result<Option<Message>, Error> {
        match self.rx.lock().await.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => Err(Error::Closed),
        }
    }
}

/// An mpsc channel factory.
#[derive(Debug, Default)]
pub struct MpscChannel {
    buffer_size: usize,
}

impl MpscChannel {
    /// Create with default buffer size (32).
    pub fn new() -> Self {
        Self { buffer_size: 32 }
    }

    /// Create with custom buffer size.
    pub fn with_buffer_size(size: usize) -> Self {
        Self { buffer_size: size }
    }
}

impl Channel for MpscChannel {
    type Sender = MpscSender;
    type Receiver = MpscReceiver;

    fn create(&self) -> (Self::Sender, Self::Receiver) {
        let (tx, rx) = mpsc::channel(self.buffer_size);
        (
            MpscSender { tx },
            MpscReceiver {
                rx: tokio::sync::Mutex::new(rx),
            },
        )
    }
}

/// A broadcast topic subscriber.
pub struct BroadcastSubscriber {
    rx: tokio::sync::Mutex<broadcast::Receiver<Message>>,
}

impl Receiver for BroadcastSubscriber {
    async fn receive(&self) -> Result<Message, Error> {
        loop {
            match self.rx.lock().await.recv().await {
                Ok(msg) => return Ok(msg),
                Err(broadcast::error::RecvError::Lagged(_)) => continue, // Skip lagged messages
                Err(broadcast::error::RecvError::Closed) => return Err(Error::Closed),
            }
        }
    }

    async fn receive_timeout(&self, timeout: Duration) -> Result<Message, Error> {
        tokio::time::timeout(timeout, self.receive())
            .await
            .map_err(|_| Error::Timeout)?
    }

    async fn try_receive(&self) -> Result<Option<Message>, Error> {
        match self.rx.lock().await.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(broadcast::error::TryRecvError::Empty) => Ok(None),
            Err(broadcast::error::TryRecvError::Lagged(_)) => Ok(None), // Treat lagged as empty
            Err(broadcast::error::TryRecvError::Closed) => Err(Error::Closed),
        }
    }
}

impl Subscriber for BroadcastSubscriber {
    async fn unsubscribe(self) -> Result<(), Error> {
        // Just drop the receiver
        Ok(())
    }
}

/// A broadcast topic.
pub struct BroadcastTopic {
    tx: broadcast::Sender<Message>,
}

impl BroadcastTopic {
    fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }
}

impl Topic for BroadcastTopic {
    type Subscriber = BroadcastSubscriber;

    async fn publish(&self, message: Message) -> Result<(), Error> {
        // It's ok if there are no receivers
        let _ = self.tx.send(message);
        Ok(())
    }

    async fn subscribe(&self) -> Result<Self::Subscriber, Error> {
        Ok(BroadcastSubscriber {
            rx: tokio::sync::Mutex::new(self.tx.subscribe()),
        })
    }
}

/// Shared topic wrapper.
#[derive(Clone)]
pub struct SharedTopic(Arc<BroadcastTopic>);

impl Topic for SharedTopic {
    type Subscriber = BroadcastSubscriber;

    async fn publish(&self, message: Message) -> Result<(), Error> {
        self.0.publish(message).await
    }

    async fn subscribe(&self) -> Result<Self::Subscriber, Error> {
        self.0.subscribe().await
    }
}

/// In-memory messaging system.
#[derive(Default)]
pub struct MemoryMessaging {
    topics: RwLock<HashMap<String, Arc<BroadcastTopic>>>,
    channel_buffer: usize,
    topic_capacity: usize,
}

impl MemoryMessaging {
    /// Create a new messaging system with default settings.
    pub fn new() -> Self {
        Self {
            topics: RwLock::new(HashMap::new()),
            channel_buffer: 32,
            topic_capacity: 64,
        }
    }

    /// Create with custom buffer sizes.
    pub fn with_config(channel_buffer: usize, topic_capacity: usize) -> Self {
        Self {
            topics: RwLock::new(HashMap::new()),
            channel_buffer,
            topic_capacity,
        }
    }
}

impl Messaging for MemoryMessaging {
    type Channel = MpscChannel;
    type Topic = SharedTopic;

    fn channel(&self) -> Self::Channel {
        MpscChannel::with_buffer_size(self.channel_buffer)
    }

    async fn topic(&self, name: &str) -> Result<Self::Topic, Error> {
        // Try read first
        {
            let topics = self.topics.read().map_err(|e| Error::Other(e.to_string()))?;
            if let Some(topic) = topics.get(name) {
                return Ok(SharedTopic(topic.clone()));
            }
        }

        // Create if not exists
        let mut topics = self.topics.write().map_err(|e| Error::Other(e.to_string()))?;
        let topic = topics
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(BroadcastTopic::new(self.topic_capacity)));
        Ok(SharedTopic(topic.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn channel_send_receive() {
        let channel = MpscChannel::new();
        let (tx, rx) = channel.create();

        tx.send(Message::new(b"hello".to_vec())).await.unwrap();
        let msg = rx.receive().await.unwrap();
        assert_eq!(msg.data, b"hello");
    }

    #[tokio::test]
    async fn channel_try_receive() {
        let channel = MpscChannel::new();
        let (tx, rx) = channel.create();

        assert!(rx.try_receive().await.unwrap().is_none());

        tx.send(Message::new(b"test".to_vec())).await.unwrap();
        assert!(rx.try_receive().await.unwrap().is_some());
    }

    #[tokio::test]
    async fn topic_pubsub() {
        let messaging = MemoryMessaging::new();
        let topic = messaging.topic("events").await.unwrap();

        let sub1 = topic.subscribe().await.unwrap();
        let sub2 = topic.subscribe().await.unwrap();

        topic
            .publish(Message::new(b"event1".to_vec()))
            .await
            .unwrap();

        let msg1 = sub1.receive().await.unwrap();
        let msg2 = sub2.receive().await.unwrap();
        assert_eq!(msg1.data, b"event1");
        assert_eq!(msg2.data, b"event1");
    }

    #[tokio::test]
    async fn message_with_metadata() {
        let msg = Message::new(b"data")
            .with_metadata("content-type", "application/json")
            .with_metadata("trace-id", "abc123");

        assert_eq!(msg.metadata.len(), 2);
        assert_eq!(msg.metadata[0], ("content-type".to_string(), "application/json".to_string()));
    }

    #[tokio::test]
    async fn receive_timeout() {
        let channel = MpscChannel::new();
        let (_tx, rx) = channel.create();

        let result = rx.receive_timeout(Duration::from_millis(10)).await;
        assert!(matches!(result, Err(Error::Timeout)));
    }
}
