#![feature(impl_trait_in_assoc_type)]

pub mod cache;
pub mod conf;
mod graceful;
mod topic;
use cache::CACHE;
pub use conf::CONFIG;
use futures::lock::Mutex;
pub use graceful::CHANNEL;
use std::io::Write;
use topic::TOPIC;
use volo::FastStr;

use std::net::SocketAddr;

struct Inner {
    inner: Mutex<Vec<volo_gen::volo::redis::RedisClient>>,
}

unsafe impl Send for Inner {}

impl Inner {
    fn new() -> Self {
        let addrs: Option<Vec<String>> = CONFIG.get_slave_addrs();
        match addrs {
            Some(addrs) => {
                let mut clients = Vec::new();
                for addr in addrs {
                    let addr: SocketAddr = addr.parse().unwrap();
                    let client = volo_gen::volo::redis::RedisClientBuilder::new("redis-client")
                        .address(addr)
                        .build();
                    clients.push(client);
                }
                Self {
                    inner: Mutex::new(clients),
                }
            }
            None => Self {
                inner: Mutex::new(Vec::new()),
            },
        }
    }

    async fn send_set(&self, key: String, value: String, ttl: Option<i32>) {
        let send = CHANNEL.send.lock().await.clone();
        let clients = self.inner.lock().await;
        for client in clients.iter() {
            let req = volo_gen::volo::redis::SetRequest {
                key: FastStr::new(key.as_str()),
                value: FastStr::new(value.as_str()),
                ttl,
            };
            let resp = client.sync(req).await;
            match resp {
                Ok(resp) => {
                    if !resp.success {
                        tracing::error!("sync failed to slave");
                    }
                }
                Err(e) => tracing::error!("{:?}", e),
            }
        }
        drop(send);
    }

    async fn send_del(&self, key: String) {
        let send = CHANNEL.send.lock().await.clone();
        let mut clients = self.inner.lock().await;
        for client in clients.iter_mut() {
            let req = volo_gen::volo::redis::DelRequest {
                key: FastStr::new(key.as_str()),
            };
            let resp = client.sync_del(req).await;
            match resp {
                Ok(resp) => {
                    if !resp.success {
                        tracing::error!("sync failed to slave");
                    }
                }
                Err(e) => tracing::error!("{:?}", e),
            }
        }
        drop(send);
    }
}

lazy_static::lazy_static! {
    static ref SLAVES: Inner = Inner::new();
}

pub struct S {
    pub file: std::sync::Mutex<std::fs::File>,
}

#[volo::async_trait]
impl volo_gen::volo::redis::Redis for S {
    async fn get(
        &self,
        _request: volo_gen::volo::redis::GetRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::GetResponse, ::volo_thrift::AnyhowError>
    {
        let key = _request.key.as_str();
        match CACHE.get(key).await {
            Some(value) => Ok(volo_gen::volo::redis::GetResponse {
                value: Some(FastStr::new(value)),
            }),
            None => Ok(volo_gen::volo::redis::GetResponse { value: None }),
        }
    }

    async fn set(
        &self,
        _request: volo_gen::volo::redis::SetRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::SetResponse, ::volo_thrift::AnyhowError>
    {
        if !CONFIG.is_master() {
            return Err(anyhow::anyhow!("Set is not allowed on slave").into());
        }
        let key = _request.key.into_string();
        let value = _request.value.into_string();
        let ttl = _request.ttl;
        CACHE.insert(key.clone(), value.clone(), ttl).await;
        match ttl {
            Some(ttl) => {
                let mut file = self.file.lock().unwrap();
                let expire_at = ttl as u128 * 1000
                    + std::time::SystemTime::now()
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis();
                let mut buf = format!(
                    "*4\n$3\nset\n${}\n{}\n${}\n{}\n",
                    key.len(),
                    key,
                    value.len(),
                    value
                );
                if ttl > 0 {
                    buf.push_str(&format!(
                        "${}\n{}\n",
                        expire_at.to_string().len(),
                        expire_at
                    ));
                }
                file.write_all(buf.as_bytes()).unwrap();
            }
            None => {
                let mut file = self.file.lock().unwrap();
                let buf = format!(
                    "*3\n$3\nset\n${}\n{}\n${}\n{}\n",
                    key.len(),
                    key,
                    value.len(),
                    value
                );
                file.write_all(buf.as_bytes()).unwrap();
            }
        }
        let _ = tokio::spawn(async move {
            SLAVES.send_set(key, value, ttl).await;
        });
        Ok(volo_gen::volo::redis::SetResponse { success: true })
    }

    async fn del(
        &self,
        _request: volo_gen::volo::redis::DelRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::DelResponse, ::volo_thrift::AnyhowError>
    {
        let key = _request.key.into_string();
        CACHE.del(&key).await;
        let mut file = self.file.lock().unwrap();
        let buf = format!("*2\n$3\ndel\n${}\n{}\n", key.len(), key);
        file.write_all(buf.as_bytes()).unwrap();
        let _ = tokio::spawn(async move {
            SLAVES.send_del(key).await;
        });
        Ok(volo_gen::volo::redis::DelResponse { success: true })
    }

    async fn subscribe(
        &self,
        _request: volo_gen::volo::redis::SubscribeRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::SubscribeResponse, ::volo_thrift::AnyhowError>
    {
        let topics = _request.topics;
        let mut res: Vec<i32> = Vec::new();
        topics.iter().for_each(|topic| {
            res.push(TOPIC.subscribe(topic.to_string()).try_into().unwrap());
        });
        Ok(volo_gen::volo::redis::SubscribeResponse { offsets: res })
    }

    async fn unsubscribe(
        &self,
        _request: volo_gen::volo::redis::UnsubscribeRequest,
    ) -> ::core::result::Result<
        volo_gen::volo::redis::UnsubscribeResponse,
        ::volo_thrift::AnyhowError,
    > {
        let topics = _request.topics;
        topics.iter().for_each(|topic| {
            TOPIC.unsubscribe(topic.to_string());
        });
        Ok(volo_gen::volo::redis::UnsubscribeResponse { success: true })
    }

    async fn publish(
        &self,
        _request: volo_gen::volo::redis::PublishRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::PublishResponse, ::volo_thrift::AnyhowError>
    {
        let topic = _request.topic.as_str();
        let message = _request.data.as_str();
        TOPIC.publish(topic.to_string(), message.to_string());
        Ok(volo_gen::volo::redis::PublishResponse { success: true })
    }

    async fn poll(
        &self,
        _request: volo_gen::volo::redis::PollRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::PollResponse, ::volo_thrift::AnyhowError>
    {
        let topics = _request.topics;
        let offsets = _request.offsets;
        // check valid
        let mut valid = true;
        offsets.iter().for_each(|offset| {
            if *offset < 0 {
                valid = false;
            }
        });
        if !valid || topics.len() != offsets.len() {
            return Ok(Default::default());
        }
        let mut messages = Vec::new();
        let mut offsets_res = Vec::new();
        for (topic, offset) in topics.iter().zip(offsets.into_iter()) {
            let (offset_res, messages_res) =
                TOPIC.poll(topic.to_string(), offset.try_into().unwrap());
            offsets_res.push(offset_res.try_into().unwrap());
            messages_res.iter().for_each(|message| {
                messages.push(volo_gen::volo::redis::Message {
                    topic: FastStr::new(topic),
                    data: FastStr::new(message),
                });
            });
        }
        Ok(volo_gen::volo::redis::PollResponse {
            messages,
            offsets: offsets_res,
        })
    }

    async fn ping(
        &self,
    ) -> ::core::result::Result<volo_gen::volo::redis::PingResponse, ::volo_thrift::AnyhowError>
    {
        Ok(volo_gen::volo::redis::PingResponse(FastStr::new("PONG")))
    }

    async fn sync(
        &self,
        _request: volo_gen::volo::redis::SetRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::SetResponse, ::volo_thrift::AnyhowError>
    {
        let key = _request.key.as_str();
        let value = _request.value.as_str();
        let ttl = _request.ttl;
        CACHE.insert(key.to_string(), value.to_string(), ttl).await;
        Ok(volo_gen::volo::redis::SetResponse { success: true })
    }

    async fn sync_del(
        &self,
        _request: volo_gen::volo::redis::DelRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::DelResponse, ::volo_thrift::AnyhowError>
    {
        let key = _request.key.as_str();
        CACHE.del(key).await;
        Ok(volo_gen::volo::redis::DelResponse { success: true })
    }
}
