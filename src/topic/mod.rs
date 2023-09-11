use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
};

lazy_static::lazy_static! {
    pub static ref TOPIC: TopicManager = TopicManager::new();
}
pub struct Topic {
    pub messages: Mutex<VecDeque<String>>,
    pub offset: usize,
    pub suscribers: usize,
}

impl Topic {
    pub fn new() -> Self {
        Self {
            messages: Mutex::new(VecDeque::new()),
            offset: 0,
            suscribers: 0,
        }
    }
}

pub struct TopicManager {
    pub topics: Mutex<HashMap<String, Topic>>,
}

impl TopicManager {
    pub fn new() -> Self {
        Self {
            topics: Mutex::new(HashMap::new()),
        }
    }

    pub fn subscribe(&self, topic_name: String) -> usize {
        let mut binding = self.topics.lock().unwrap();
        let topic = binding.entry(topic_name).or_insert(Topic::new());
        let len = topic.messages.lock().unwrap().len();
        len
    }

    pub fn unsubscribe(&self, topic_name: String) {
        match self.topics.lock().unwrap().get_mut(&topic_name) {
            Some(topic) => {
                topic.suscribers -= 1;
                if topic.suscribers == 0 {
                    self.topics.lock().unwrap().remove(&topic_name);
                }
            }
            None => {}
        }
    }

    pub fn publish(&self, topic_name: String, message: String) {
        match self.topics.lock().unwrap().get_mut(&topic_name) {
            Some(topic) => {
                topic.messages.lock().unwrap().push_back(message);
            }
            None => {}
        }
    }

    // must subscribe first
    pub fn poll(&self, topic_name: String, offset: usize) -> (usize, Vec<String>) {
        let mut binding = self.topics.lock().unwrap();
        let topic = binding.entry(topic_name).or_insert(Topic::new());
        let messages = topic.messages.lock().unwrap();
        let len = messages.len();
        let mut result = Vec::new();
        for i in offset..len {
            result.push(messages[i].clone());
        }
        topic.offset = len;
        (len, result)
    }
}
