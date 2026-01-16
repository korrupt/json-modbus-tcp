use std::{
    fs::{self}, net::SocketAddr, path::PathBuf, str::FromStr
};

use clap::{Parser, ValueHint};
use fern::Dispatch;
use serde_with::{serde_as, DisplayFromStr};
use validation::{parse_whitelist};
use log::{LevelFilter, info};
use serde::{Serialize, Deserialize};

mod register;
mod pack;
mod register_manager;
mod server;
mod service;
mod util;
mod validation;

use crate::{register::{RegisterDefinition, deserialize_register_definintions}, server::{FileFormat, OutputConfig, ServerConfig}};

fn default_target() -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], 4000))
}

fn default_loglevel() -> LevelFilter {
    LevelFilter::Info
}

fn parse_registers(s: &str) -> Result<RegisterDefinition, String> {
    RegisterDefinition::from_str(s)
}

fn default_whitelist() -> Vec<String> {
    vec![
        "0.0.0.0:rw".into(),
        "127.0.0.1:rw".into()
    ]
}



#[serde_as]
#[derive(Parser, Serialize, Deserialize, Debug)]
pub struct Args {

    /// Optional config path
    #[serde(skip_serializing)]
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    config: Option<PathBuf>,

    /// Output path
    #[arg(short, long)]
    output: Option<OutputConfig>,

    /// target IP address and port
    #[arg(short, default_value_t = default_target())]
    #[serde(default = "default_target")]
    target: SocketAddr,
    
    /// Log Level (off, error, info, warn, trace)
    #[arg(short, default_value_t = LevelFilter::Info)]
    #[serde(default = "default_loglevel")]
    #[serde_as(as = "DisplayFromStr")]
    loglevel: log::LevelFilter,

    /// CIDR Whitelist (r/w/rw) (comma separated)
    #[arg(short = 'W', default_value =  "0.0.0.0:rw,127.0.0.1:rw", use_value_delimiter = true)]
    #[serde(default = "default_whitelist")]
    global_whitelist: Vec<String>,

    /// Define registers. [offset][/type]:[default_value]
    #[arg(short, use_value_delimiter = true, value_parser = parse_registers )]
    #[serde(deserialize_with = "deserialize_register_definintions")]
    registers: Vec<RegisterDefinition>
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();


    let config: Args = if let Some(path) = args.config {
            if !path.exists() { return Err("File doesnt exist".into()) }

            let format = path
                .extension()
                .and_then(|e| e.to_str())
                .ok_or("Missing config extension")
                .and_then(|e| FileFormat::from_extension_str(e).ok_or("Invalid extension, only .yaml/.yml and json files are supported"))?;                

            let content = fs::read_to_string(path)?;

            match format {
                FileFormat::JSON => serde_json::from_str(&content)?,
                FileFormat::YAML => serde_yaml::from_str(&content)?
            }
        } else {
            args
        };

        
    let global_whitelist = parse_whitelist(&config.global_whitelist)?;
    let Args { target, loglevel, output, registers, .. } = config;
    
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(loglevel)
        .level_for("tokio_modbus", LevelFilter::Off)
        .chain(std::io::stdout())
        .apply().unwrap();

    info!("Starting with logging set to {}", config.loglevel);

    server::server_context(ServerConfig {
        socket_addr: target,
        register_definitions: registers,
        global_whitelist,
        output,
    }).await?;

    Ok(())
}

