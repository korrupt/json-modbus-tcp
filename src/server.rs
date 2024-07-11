use std::thread;
use std::time::Duration;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

use tokio_modbus::server::tcp::{accept_tcp_connection, Server};
use crate::json;
use crate::register_manager::RegisterManager;
use crate::service::ModbusService;

pub async fn server_context(socket_addr: SocketAddr, update_frequency: Duration) -> anyhow::Result<()> {
    println!("Starting up server on {socket_addr}");

    let listener = TcpListener::bind(socket_addr).await?;
    let server = Server::new(listener);

    let manager = Arc::new(
        match json::load("data.json")
            .and_then(|v| RegisterManager::from_json(v)) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Failed to loading json. Using empty registers. Error: {e}");
                    RegisterManager::new()
                }
            }
    );

    let new_service = |_addr| {
        Ok(Some(ModbusService::new(manager.clone())))
    };

    let on_connected = |stream, socket_addr| async move {
        accept_tcp_connection(stream, socket_addr, &new_service)
    };

    let on_process_error = |err| {
        eprintln!("{err}");
    };

    new_service(socket_addr)?;

    let persistence_clone = manager.clone();
    let (tx_stop, rx_stop) = std::sync::mpsc::channel::<()>();

    let persistence_thread = thread::spawn(move || {
        loop {
            if rx_stop.try_recv().is_ok() {
                break;
            }

            thread::sleep(update_frequency);
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