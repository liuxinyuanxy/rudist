use std::io::{Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddrV4, TcpListener, TcpStream};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use thread::spawn;
use Ordering::Relaxed;
use std::cell::RefCell;
use anyhow::Result;
use tracing::{error, info};
use crate::ProxyConfig;
use crate::Cluster;
pub struct ProxyService {
    name: Arc<String>,
    listen: Arc<SocketAddrV4>,
    target: Arc<SocketAddrV4>,
    connection: Arc<Option<Connection>>,
}

pub struct Connection {
    name: Arc<String>,
    listen_stream: Arc<TcpStream>,
    target_stream: Arc<TcpStream>,
    rx_bytes: AtomicU32,
    tx_bytes: AtomicU32,
}

impl Connection {
    pub fn new(
        pre: Arc<Option<Connection>>,
        name: Arc<String>,
        listen_stream: TcpStream,
        target: Arc<SocketAddrV4>,
    ) -> Result<Connection> {
        let target_stream = TcpStream::connect(*target).map_err(|error| {
            error!("{} failed to connect target {}", name, target);
            let _ = listen_stream.shutdown(Shutdown::Both);
            return error;
        })?;
        info!(
            "{} connection established from {}",
            name,
            listen_stream.peer_addr()?
        );
        info!(
            "{} connection established to {}",
            name,
            target_stream.peer_addr()?
        );

        let mut rx = 0;
        if let Some(con) = pre.clone().as_ref() {
            rx = con.rx_bytes.load(Relaxed);
        }
        let mut tx = 0;
        if let Some(con) = pre.clone().as_ref() {
            tx = con.tx_bytes.load(Relaxed);
        }
       
        let con = Connection {
            name,
            listen_stream: Arc::new(listen_stream),
            target_stream: Arc::new(target_stream),
            rx_bytes: AtomicU32::new(rx),
            tx_bytes: AtomicU32::new(tx),
        };
        return Ok(con);
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let name = self.name.clone();
        // close previous steam
        self.target_stream.shutdown(Shutdown::Both).unwrap();
        info!(
            "{} shutdown previous source stream {:?}",
            name.clone(),
            self.target_stream.clone()
        );

        self.listen_stream.shutdown(Shutdown::Both).unwrap();
        info!(
            "{} shutdown previous source stream {:?}",
            name.clone(),
            self.listen_stream.clone()
        );
    }
}

impl ProxyService {
    pub fn new(config: Rc<ProxyConfig>) -> Result<ProxyService> {
        let listen_socket = SocketAddrV4::new(Ipv4Addr::from_str("127.0.0.1")?, config.listen);
        let target_socket = SocketAddrV4::from_str(config.target.as_str())?;
        Ok(ProxyService {
            name: Arc::new(config.name.clone()),
            listen: Arc::new(listen_socket),
            target: Arc::new(target_socket),
            connection: Arc::new(None),
        })
    }
    // start proxy service
    pub fn run(self) -> JoinHandle<()> {
        let name = self.name.clone();
        let listen = self.listen.clone();
        let target = self.target.clone();
        info!("{:?} listen: {}, target:{} ",self.name, listen, target);
        spawn(move || {
            let listener = TcpListener::bind(*listen).expect("Failed start Listener");
            for incoming in listener.incoming() {
                info!("{} connection incoming", name.clone());
                let pre_connection = self.connection.clone();
                let name = name.clone();
                match incoming {
                    Ok(stream) => {
                        let connection =
                            Connection::new(pre_connection, name.clone(), stream, target.clone());
                        if connection.is_ok() {
                            let connection = Arc::new(connection.unwrap());
                            info!("{} connection established", name.clone());
                            let e = transfer_stream(connection.clone());
                            if e.is_err() {
                                error!(
                                    "{} copy between listen and target failed, {:?}",
                                    name.clone(),
                                    e
                                );
                            }
                        } else {
                            error!(
                                "{} connection create failed, {:?}",
                                name.clone(),
                                connection.err().unwrap()
                            )
                        }
                    }
                    Err(e) => {
                        error!("{} connect source failed, {:?}", name.clone(), e);
                    }
                }
            }
        })
    }
}
fn transfer_stream(con: Arc<Connection>) -> Result<()> {
    let request = con.clone();
    let response = con.clone();

    let name = con.name.clone();
    spawn(move || {
        let mut listen = request.listen_stream.as_ref();
        let mut target = request.target_stream.as_ref();
        let mut buf = vec![0; 1024];
        loop {
            match listen.read(&mut buf) {
                //0 if the read half of the connection has been closed.
                Ok(0) => {
                    info!("connection close by peer");
                    return;
                }
                //n if the read half of the connection has been closed and n bytes were read.
                Ok(n) => {
                    // Copy the data back to socket
                    request.rx_bytes.fetch_add(n as u32, Relaxed);
                    info!(
                        "{} traffic rx={}, tx={}, {}",
                        name.clone(),
                        request.rx_bytes.load(Relaxed),
                        request.tx_bytes.load(Relaxed),
                        n
                    );
                    if target.write_all(&buf[..n]).is_err() {
                        info!("socket error");
                        return;
                    }
                }
                Err(_) => {
                    info!("Unexpected socket error");
                    return;
                }
            }
        }
    });
    let name = con.name.clone();
    spawn(move || {
        let mut listen = response.listen_stream.as_ref();
        let mut target = response.target_stream.as_ref();
        let mut buf = vec![0; 1024];
        loop {
            match target.read(&mut buf) {
                // Return value of `Ok(0)` signifies that the remote has
                // closed
                Ok(0) => {
                    info!("connection close by peer");
                    return;
                }
                Ok(n) => {
                    // Copy the data back to socket
                    response.tx_bytes.fetch_add(n as u32, Relaxed);
                    info!(
                        "{} traffic rx={}, tx={}, {}",
                        name.clone(),
                        response.rx_bytes.load(Relaxed),
                        response.tx_bytes.load(Relaxed),
                        n
                    );
                    if listen.write_all(&buf[..n]).is_err() {
                        info!("socket error");
                        return;
                    }
                }
                Err(_) => {
                    info!("Unexpected socket error");
                    return;
                }
            }
        }
    });
    Ok(())
}
