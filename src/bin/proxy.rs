#![feature(impl_trait_in_assoc_type)]

mod middleware;
use middleware::{CheckLayer, LogLayer};
use miniredis::CHANNEL;
use miniredis::CONFIG;
use miniredis::P;
use std::net::SocketAddr;

#[volo::main]
async fn main() {
    // tracing_subscriber::fmt::init();
    let settings = config::Config::builder()
        .add_source(config::File::with_name("src/redis.toml"))
        .build()
        .unwrap();
    let addr: String = settings.get("proxy_addr").unwrap();
    let addr: SocketAddr = addr.parse().unwrap();
    let addr = volo::net::Address::from(addr);

    volo_gen::volo::redis::ProxyServer::new(P)
        .layer_front(LogLayer)
        .layer_front(CheckLayer)
        .run(addr)
        .await
        .unwrap();

    *CHANNEL.send.lock().await = None;
    let _ = CHANNEL.recv.lock().await.recv().await;
}
