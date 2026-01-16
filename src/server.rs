use std::collections::{BTreeMap};
use std::path::PathBuf;
use std::str::FromStr;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::{net::SocketAddr, sync::Arc};
use humantime_serde::re::humantime;
use log::{error, info};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::net::TcpListener;

use tokio_modbus::server::tcp::{accept_tcp_connection, Server};
use crate::register::{Register, RegisterDefinition};
use crate::register_manager::{ManagerError, RegisterManager};
use crate::service::ModbusService;
use crate::validation::Whitelist;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Io: {0}")]
    Io(#[from] std::io::Error),
    #[error("Server: {0}")]
    Manager(#[from] ManagerError),
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FileFormat {
    YAML,
    JSON
}

impl FileFormat {
    pub fn from_extension_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "yaml" | "yml" => Some(Self::YAML),
            "json" => Some(Self::JSON),
            _ => None
        }
    }
}

fn default_duration() -> Duration {
    Duration::from_secs(1)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_duration", with = "humantime_serde")]
    update_frequency: Duration,
    path: PathBuf,
    format: FileFormat,
}

impl FromStr for OutputConfig {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (path_str, update_freq_str) = s.split_once(",").unwrap_or((&s, "2s"));

        let path = PathBuf::from(path_str);
        let format = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or("Missing config extension")
            .and_then(|e| FileFormat::from_extension_str(e).ok_or("Invalid extension, only .yaml/.yml and json files are supported"))?;   

        let update_frequency = humantime::parse_duration(update_freq_str).map_err(|e| e.to_string())?;


        Ok(OutputConfig { update_frequency, path, format })
    }
}

pub struct ServerConfig {
    pub socket_addr: SocketAddr,
    pub global_whitelist: Whitelist,
    pub register_definitions: Vec<RegisterDefinition>,
    pub output: Option<OutputConfig>,
}

pub async fn server_context(config: ServerConfig) -> Result<(), ServerError> {
    let manager = Arc::new(RegisterManager::try_from_definitions(&config.register_definitions)?); 
    
    let listener = TcpListener::bind(config.socket_addr).await?;
    info!("Server listening on {}", config.socket_addr);

    let whitelist = config.global_whitelist;
    
    let server = Server::new(listener);

    let new_service = |addr: SocketAddr| {
        Ok(Some(ModbusService::new(manager.clone(), addr, whitelist.clone())))
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

    // No need for serde_as / flatten magic
    #[derive(Serialize)]
    struct Registers<'a> {
        // BTreeMap serializes sorted by key – clean & predictable output
        registers: &'a BTreeMap<String, String>,
    }

    let persistence_thread: JoinHandle<Result<(), String>> = thread::spawn(move || {
        let mut registers=  BTreeMap::new();
      
        if let Some(output) = config.output {
            loop {
                if rx_stop.try_recv().is_ok() {
                    break;
                }

                let snapshot: Register = persistence_clone.snapshot().into();
                registers.clear();

                for def in &config.register_definitions {
                    let words: Vec<u16> = def.to_range()
                            .into_iter()
                            .map(|offset| snapshot[&offset])
                            .collect();
                
                    let value_str = def.data_type.payload_str(&words);
                    let key = format!("{}/{}", def.offset, def.data_type.to_char());

                    registers.insert(key, value_str);
                }

                let content = match output.format {
                    FileFormat::JSON => serde_json::to_string_pretty(&Registers { registers: &registers })
                        .map_err(|e| e.to_string())?,
                    FileFormat::YAML => serde_yaml::to_string(&Registers { registers: &registers })
                        .map_err(|e| e.to_string())?,
                };

                std::fs::write(&output.path, content).map_err(|e| e.to_string())?;
                
                thread::sleep(output.update_frequency); 
            }
        }

        Ok(())
    });
    
    server.serve(&on_connected, on_process_error).await?;
    tx_stop.send(()).unwrap();
    persistence_thread.join().unwrap().unwrap();

    Ok(())
}