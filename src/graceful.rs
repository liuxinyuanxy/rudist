use lazy_static::lazy_static;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;

pub struct MpscChannel {
    pub send: Mutex<Option<Sender<()>>>,
    pub recv: Mutex<Receiver<()>>,
}

lazy_static! {
    pub static ref CHANNEL: MpscChannel = {
        let (send, recv): (Sender<()>, Receiver<()>) = channel(1);
        MpscChannel {
            send: Mutex::new(Some(send)),
            recv: Mutex::new(recv),
        }
    };
}
