use std::thread;
use std::time::Duration;
use std::{net::SocketAddr, sync::Arc};
use ipnetwork::IpNetwork;
use tokio::net::TcpListener;

use tokio_modbus::server::tcp::{accept_tcp_connection, Server};
use crate::json;
use crate::register_manager::RegisterManager;
use crate::service::ModbusService;


pub struct ServerConfig {
    pub socket_addr: SocketAddr,
    pub update_frequency: Duration,
    pub debug: bool,
    pub read_whitelist: Option<Vec<IpNetwork>>,
    pub write_whitelist: Option<Vec<IpNetwork>>,
}

pub async fn server_context(config: ServerConfig) -> anyhow::Result<()> {
    println!("Starting up server on {}", config.socket_addr.to_string());

    let listener = TcpListener::bind(config.socket_addr).await?;

    let manager = Arc::new(
        match json::load("data.json")
            .and_then(|v| RegisterManager::from_json(v, config.debug)) {
                Ok(v) => v,
                Err(e) => {
                    println!("Failed to loading json. Using empty registers. Error: {e}");
                    RegisterManager::new(config.debug)
                }
            }
    );

    let server = Server::new(listener);

    let new_service = |addr: SocketAddr| {
        Ok(Some(ModbusService::new(manager.clone(), addr.ip(), config.read_whitelist.clone(), config.write_whitelist.clone())))
    };

    let on_connected = |stream, socket_addr: SocketAddr| async move {
        if config.debug {
            println!("New connection: {}", socket_addr.ip());
        }
        
        accept_tcp_connection(stream, socket_addr, &new_service)
    };

    let on_process_error = |err| {
        eprintln!("{err}");
    };    

    new_service(config.socket_addr)?;

    let persistence_clone = manager.clone();
    let (tx_stop, rx_stop) = std::sync::mpsc::channel::<()>();

    let persistence_thread = thread::spawn(move || {
        loop {
            if rx_stop.try_recv().is_ok() {
                break;
            }

            thread::sleep(config.update_frequency);
            if let Err(e) = persistence_clone.update_persistence() {
                eprint!("Error updating persistence: {:?}", e);
            }
        }
    });
    
    server.serve(&on_connected, on_process_error).await?;
    tx_stop.send(()).unwrap();
    persistence_thread.join().unwrap();

    Ok(())
}