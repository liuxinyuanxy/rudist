#![feature(impl_trait_in_assoc_type)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(unboxed_closures)]
pub mod cache;
pub mod conf;
mod graceful;
mod topic;
mod transaction;
mod utils;
use cache::CACHE;
// mod proxy_service;
pub use conf::CONFIG;
pub use conf::FILE;
pub use graceful::CHANNEL;
use std::net::SocketAddr;
use tokio::sync::Mutex;
use topic::TOPIC;
pub use transaction::TRANSACTION;
use volo::FastStr;

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

pub struct S;

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
        FILE.write_to_file_new_thread(
            utils::set_to_string(&key, &value, ttl).await,
            CHANNEL.send.lock().await.clone(),
        )
        .await;

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
        if !CONFIG.is_master() {
            return Err(anyhow::anyhow!("Del is not allowed on slave").into());
        }
        let key = _request.key.into_string();
        CACHE.del(&key).await;
        FILE.write_to_file_new_thread(
            utils::del_to_string(&key).await,
            CHANNEL.send.lock().await.clone(),
        )
        .await;
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

    async fn multi(
        &self,
    ) -> ::core::result::Result<volo_gen::volo::redis::MultiResponse, ::volo_thrift::AnyhowError>
    {
        let transaction_id = TRANSACTION.new_transaction().await;
        Ok(volo_gen::volo::redis::MultiResponse(FastStr::new(
            transaction_id,
        )))
    }

    async fn watch(
        &self,
        _request: volo_gen::volo::redis::WatchRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::BoolResponse, ::volo_thrift::AnyhowError>
    {
        let transaction_id = _request.id.to_string();
        let key = _request.key.to_string();
        CACHE.watch(&key, transaction_id).await;
        Ok(volo_gen::volo::redis::BoolResponse(true))
    }

    async fn exec(
        &self,
        _request: volo_gen::volo::redis::ExecRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::ExecResponse, ::volo_thrift::AnyhowError>
    {
        let transaction_id = _request.0.to_string();
        let res = TRANSACTION.exec(&transaction_id).await;
        match res {
            Some(value) => Ok(volo_gen::volo::redis::ExecResponse {
                valid: true,
                results: value
                    .into_iter()
                    .map(|x| match x {
                        Some(v) => FastStr::new(v),
                        None => FastStr::new("key not found".to_string()),
                    })
                    .collect(),
            }),
            None => Ok(volo_gen::volo::redis::ExecResponse {
                valid: false,
                results: Vec::new(),
            }),
        }
    }

    async fn trget(
        &self,
        _request: volo_gen::volo::redis::TrGetRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::BoolResponse, ::volo_thrift::AnyhowError>
    {
        let transaction_id = _request.id.to_string();
        let key = _request.key.to_string();
        TRANSACTION
            .add_command(&transaction_id, &utils::get_to_string(&key).await)
            .await;
        Ok(volo_gen::volo::redis::BoolResponse(true))
    }
    async fn trset(
        &self,
        _request: volo_gen::volo::redis::TrSetRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::BoolResponse, ::volo_thrift::AnyhowError>
    {
        let transaction_id = _request.id.to_string();
        let key = _request.key.into_string();
        let value = _request.value.into_string();
        let ttl = _request.ttl;
        TRANSACTION
            .add_command(
                &transaction_id,
                &utils::set_to_string(&key, &value, ttl).await,
            )
            .await;
        Ok(volo_gen::volo::redis::BoolResponse(true))
    }
    async fn trdel(
        &self,
        _request: volo_gen::volo::redis::TrDelRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::BoolResponse, ::volo_thrift::AnyhowError>
    {
        let transaction_id = _request.id.to_string();
        let key = _request.key.to_string();
        TRANSACTION
            .add_command(&transaction_id, &utils::del_to_string(&key).await)
            .await;
        Ok(volo_gen::volo::redis::BoolResponse(true))
    }
}

struct ProxyInner {
    inner: Mutex<Vec<volo_gen::volo::redis::RedisClient>>,
}

unsafe impl Send for ProxyInner {}

impl ProxyInner {
    fn new() -> Self {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("src/redis.toml"))
            .build()
            .unwrap();
        let addrs: Vec<String> = settings.get("addrs").unwrap();
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
}

lazy_static::lazy_static! {
    static ref SLOTS: ProxyInner = ProxyInner::new();
}
pub struct P;
#[volo::async_trait]
impl volo_gen::volo::redis::Proxy for P {
    async fn get(
        &self,
        _request: volo_gen::volo::redis::GetRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::GetResponse, ::volo_thrift::AnyhowError>
    {
        let key = _request.key.as_str();
        let clients = SLOTS.inner.lock().await;
        const X25: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_IBM_SDLC);
        let mut hasher = X25.digest();
        hasher.update(key.as_bytes());
        let hash = hasher.finalize();
        let slot = hash as usize % 3;
        let resp = clients[slot]
            .get(volo_gen::volo::redis::GetRequest {
                key: FastStr::new(key),
            })
            .await?;
        Ok(resp)
    }

    async fn set(
        &self,
        _request: volo_gen::volo::redis::SetRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::SetResponse, ::volo_thrift::AnyhowError>
    {
        let key = _request.key.into_string();
        let value = _request.value.into_string();
        let ttl = _request.ttl;
        let clients = SLOTS.inner.lock().await;
        const X25: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_IBM_SDLC);
        let mut hasher = X25.digest();
        hasher.update(key.as_bytes());
        let hash = hasher.finalize();
        let slot = hash as usize % 3;
        clients[slot]
            .set(volo_gen::volo::redis::SetRequest {
                key: FastStr::new(key.as_str()),
                value: FastStr::new(value.as_str()),
                ttl,
            })
            .await?;
        Ok(volo_gen::volo::redis::SetResponse { success: true })
    }

    async fn del(
        &self,
        _request: volo_gen::volo::redis::DelRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::DelResponse, ::volo_thrift::AnyhowError>
    {
        let key = _request.key.into_string();
        let clients = SLOTS.inner.lock().await;
        const X25: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_IBM_SDLC);
        let mut hasher = X25.digest();
        hasher.update(key.as_bytes());
        let hash = hasher.finalize();
        let slot = hash as usize % 16384;
        clients[slot]
            .del(volo_gen::volo::redis::DelRequest {
                key: FastStr::new(key.as_str()),
            })
            .await?;
        Ok(volo_gen::volo::redis::DelResponse { success: true })
    }

    async fn ping(
        &self,
    ) -> ::core::result::Result<volo_gen::volo::redis::PingResponse, ::volo_thrift::AnyhowError>
    {
        Ok(volo_gen::volo::redis::PingResponse(FastStr::new("PONG")))
    }
}
