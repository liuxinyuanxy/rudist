#![feature(impl_trait_in_assoc_type)]

mod middleware;
use middleware::{CheckLayer, LogLayer};
use miniredis::S;
use std::net::SocketAddr;
#[volo::main]
async fn main() {
    // set the log mod as debug
    tracing_subscriber::fmt::init();

    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    volo_gen::volo::redis::RedisServer::new(S)
        .layer_front(LogLayer)
        .layer_front(CheckLayer)
        .run(addr)
        .await
        .unwrap();
}
