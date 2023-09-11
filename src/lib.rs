#![feature(impl_trait_in_assoc_type)]

mod cache;
use cache::CACHE;
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
        Ok(Default::default())
    }
    async fn publish(
        &self,
        _request: volo_gen::volo::redis::PublishRequest,
    ) -> ::core::result::Result<volo_gen::volo::redis::PublishResponse, ::volo_thrift::AnyhowError>
    {
        Ok(Default::default())
    }
    async fn ping(
        &self,
    ) -> ::core::result::Result<volo_gen::volo::redis::PingResponse, ::volo_thrift::AnyhowError>
    {
        Ok(volo_gen::volo::redis::PingResponse(FastStr::new("PONG")))
    }
}
