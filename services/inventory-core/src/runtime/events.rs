use tokio::sync::broadcast;

use crate::runtime::contracts::EventEnvelope;

#[derive(Debug, Clone)]
pub struct InProcessEventStream {
    sender: broadcast::Sender<EventEnvelope>,
}

impl InProcessEventStream {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn publish(&self, event: EventEnvelope) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.sender.subscribe()
    }
}
