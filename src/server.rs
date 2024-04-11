use std::{net::SocketAddr, process::exit};
use tokio::net::TcpListener;

use tokio_modbus::server::tcp::{accept_tcp_connection, Server};

use crate::json::{self, JsonError};

use super::service::BatteryService;

pub async fn server_context(socket_addr: SocketAddr) -> anyhow::Result<()> {
    println!("Starting up server on {socket_addr}");

    let listener = TcpListener::bind(socket_addr).await?;

    let server = Server::new(listener);
    
    let new_service = |_addr| {
        match json::load_json("data.json") {
            Ok(data) => {
                let service = BatteryService::try_from_json(data).expect("Error loading json");
                return Ok(Some(service))
            },
            Err(JsonError::NoFile) => {
                println!("No data.json file, starting with default data");
                return Ok(Some(BatteryService::new()))
            },
            Err(e) => {
                eprint!("{e}");
                exit(-1);
            }
        }
    };

    let on_connected = |stream, socket_addr| async move {
        accept_tcp_connection(stream, socket_addr, new_service)
    };

    let on_process_error = |err| {
        eprintln!("{err}");
    };

    new_service(socket_addr)?;
    
    server.serve(&on_connected, on_process_error).await?;

    Ok(())
}