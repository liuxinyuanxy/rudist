use std::collections::HashMap;

lazy_static::lazy_static! {
    pub static ref TOPIC: topic_manager = topic_manager::new();
}
pub struct Topic {
    pub name: String,
    pub sender: async_channel::Sender<String>,
    pub receiver: async_channel::Receiver<String>,
}

impl Topic {
    pub fn new(name: String) -> Self {
        let (sender, receiver) = async_channel::bounded(1024);
        Self {
            name,
            sender,
            receiver,
        }
    }
}

pub struct topic_manager {
    pub topics: HashMap<String, Topic>,
}

impl topic_manager {
    pub fn new() -> Self {
        Self {
            topics: HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, topic_name: String) -> async_channel::Receiver<String> {
        let topic = self
            .topics
            .entry(topic_name.clone())
            .or_insert(Topic::new(topic_name));
        topic.receiver.clone()
    }

    pub fn publish(&mut self, topic_name: String, message: String) {
        let topic = self
            .topics
            .entry(topic_name.clone())
            .or_insert(Topic::new(topic_name));
        topic.sender.try_send(message).unwrap();
    }
}
