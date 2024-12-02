use std::thread;
use std::time::Duration;
use std::{net::SocketAddr, sync::Arc};
use ipnetwork::IpNetwork;
use log::{error, info};
use tokio::net::TcpListener;

use tokio_modbus::server::tcp::{accept_tcp_connection, Server};
use crate::json;
use crate::register_manager::RegisterManager;
use crate::service::ModbusService;


pub struct ServerConfig {
    pub socket_addr: SocketAddr,
    pub update_frequency: Duration,
    pub read_whitelist: Option<Vec<IpNetwork>>,
    pub write_whitelist: Option<Vec<IpNetwork>>,
}

pub async fn server_context(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    info!("Server listening on {}", config.socket_addr);

    let listener = TcpListener::bind(config.socket_addr).await?;

    let manager = Arc::new(
        match json::load("data.json")
            .and_then(|v| RegisterManager::from_json(v)) {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to loading json: {e}");
                    return Err("Failed to load json".into())
                }
            }
    );

    let server = Server::new(listener);

    let new_service = |addr: SocketAddr| {
        Ok(Some(ModbusService::new(manager.clone(), addr, config.read_whitelist.clone(), config.write_whitelist.clone())))
    };

    let on_connected = |stream, socket_addr: SocketAddr| async move {
        accept_tcp_connection(stream, socket_addr, &new_service)
    };

    let on_process_error = |err| {
        error!("{err}");
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
                error!("Error updating persistence: {:?}", e);
            }
        }
    });
    
    server.serve(&on_connected, on_process_error).await?;
    tx_stop.send(()).unwrap();
    persistence_thread.join().unwrap();

    Ok(())
}