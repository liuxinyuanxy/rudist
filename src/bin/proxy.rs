extern crate core;
extern crate serde_derive;
extern crate toml;

use anyhow::{Context, Result};
use clap::Parser;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use tracing::{debug, error, info, Level};

use miniredis::{ClusterProxyConfig, ProxyArgs, ProxyService, Cluster};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args: ProxyArgs = ProxyArgs::parse();
    info!("config file: {:?}", args.config);
    //read file from confg
    let config_value = read_config(args.config)?;
    //parse toml
    let proxy_config: ClusterProxyConfig = toml::from_str(&config_value)?;
    debug!("config: {:?}", proxy_config);
    let mut handles = Vec::new();
    match proxy_config.proxy {
        Some(proxy_vec) => {
            for proxy in proxy_vec {
                let proxy_rf = Rc::new(proxy);
                // if enable is not false
                if proxy_rf.enable != Some(false) {
                    let service = ProxyService::new(proxy_rf.clone())
                        .context(format!("invalid value in {:?}", proxy_rf))?;
                    handles.push(service.run());
                }
                //if enable is true
                else {
                    info!("proxy {:?} is disabled.", proxy_rf);
                }
            }
        }
        None => {
            info!("Please setup config file.")
        }
    };
    for handle in handles {
        let _ = handle.join().expect("exit.");
    }
    Ok(())
}

fn read_config(config_path: Option<String>) -> Result<String> {
    let mut config_value = String::new();
    match config_path {
        Some(path) => {
            // if file specified
            let mut file = File::open(path)?;
            file.read_to_string(&mut config_value)?;
        }
        None => {
            //default is proxy.toml
            let mut file = File::open("proxy.toml")?;
            file.read_to_string(&mut config_value)?;
        }
    }
    return Ok(config_value);
}
