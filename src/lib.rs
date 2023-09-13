#![feature(impl_trait_in_assoc_type)]

mod cache;
mod conf;
mod topic;
use cache::CACHE;
pub use conf::CONFIG;
use topic::TOPIC;
use volo::FastStr;
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
        let key = _request.key.as_str();
        let value = _request.value.as_str();
        let ttl = _request.ttl;
        CACHE.insert(key.to_string(), value.to_string(), ttl).await;
        Ok(volo_gen::volo::redis::SetResponse { success: true })
    }

    async fn del(
        &self,
        _request: volo_gen::volo::redis::DelRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::DelResponse, ::volo_thrift::AnyhowError>
    {
        let key = _request.key.as_str();
        CACHE.del(key).await;
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
}
