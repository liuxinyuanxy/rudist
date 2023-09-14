use crate::TRANSACTION;
use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};
lazy_static::lazy_static! {
    pub static ref CACHE: Cache = Cache::new(1024);
}

trait CanExpire {
    fn is_expired(&self) -> bool;
}

struct CacheInner<K, V: CanExpire> {
    data: HashMap<K, V>,
}

impl<K, V> CacheInner<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: CanExpire,
{
    fn flush(&mut self) {
        self.data.retain(|_, v| !v.is_expired());
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashMap::with_capacity(capacity),
        }
    }

    fn cache_set(&mut self, k: K, v: V) {
        self.data.insert(k, v);
        self.flush();
    }

    fn cache_get<Q>(&mut self, k: &Q) -> Option<&V>
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq + ?Sized,
    {
        match self.data.get(k) {
            Some(v) => {
                if v.is_expired() {
                    None
                } else {
                    Some(v)
                }
            }
            None => None,
        }
    }

    fn cache_remove<Q>(&mut self, k: &Q)
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq + ?Sized,
    {
        self.data.remove(k);
    }
}

struct Entity {
    value: String,
    expired_at: Option<Instant>,
    watched: Mutex<Vec<String>>,
}

impl Entity {
    fn watch(&self) {
        let watched = self.watched.lock().unwrap();
        for watcher in watched.iter() {
            let id = watcher.clone();
            let _ = tokio::spawn(async move { TRANSACTION.set_invalid(&id).await });
        }
    }
}

impl CanExpire for Entity {
    fn is_expired(&self) -> bool {
        match self.expired_at {
            Some(expired_at) => expired_at < Instant::now(),
            None => false,
        }
    }
}

pub struct Cache {
    cache: Mutex<CacheInner<String, Entity>>,
}

impl Cache {
    pub fn new(capacity: usize) -> Self {
        let cache = CacheInner::with_capacity(capacity);
        tracing::debug!("created cache with capacity: {}", capacity);
        Self {
            cache: Mutex::new(cache),
        }
    }

    pub async fn insert(&self, key: String, value: String, ttl: Option<i32>) {
        {
            let mut cache = self.cache.lock().unwrap();
            let entity = cache.cache_get(&key);
            match entity {
                Some(entity) => entity.watch(),
                None => (),
            }
        }
        let entity = Entity {
            value,
            expired_at: match ttl {
                Some(ttl) => Some(Instant::now() + Duration::from_secs(ttl as u64)),
                None => None,
            },
            watched: Mutex::new(Vec::new()),
        };
        tracing::debug!("try to insert key: {}", key);
        self.cache.lock().unwrap().cache_set(key, entity);
        tracing::debug!("key inserted")
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        tracing::debug!("try to get key: {}", key);
        self.cache
            .lock()
            .unwrap()
            .cache_get(key)
            .map(|entity| entity.value.clone())
    }

    pub async fn del(&self, key: &str) {
        tracing::debug!("try to del key: {}", key);
        {
            let mut cache = self.cache.lock().unwrap();
            let entity = cache.cache_get(key);
            match entity {
                Some(entity) => entity.watch(),
                None => (),
            }
        }
        self.cache.lock().unwrap().cache_remove(key);
        tracing::debug!("key deleted")
    }

    pub async fn watch(&self, key: &str, watcher: String) {
        tracing::debug!("try to watch key: {}", key);
        let mut cache = self.cache.lock().unwrap();
        let entity = cache.cache_get(key).unwrap();
        let mut watched = entity.watched.lock().unwrap();
        watched.push(watcher);
        tracing::debug!("key watched")
    }
}
