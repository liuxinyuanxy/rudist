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
    let addr: SocketAddr = CONFIG.get_my_addr().parse().unwrap();
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
