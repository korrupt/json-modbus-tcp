use std::{
    net::SocketAddr,
    time::Duration,
};

use clap::Parser;
use fern::Dispatch;
use log::LevelFilter;
use server::ServerConfig;
use validation::{validate_time, parse_whitelist};

mod json;
mod pack;
mod register_manager;
mod server;
mod service;
mod util;
mod validation;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// target IP address and port
    #[arg(default_value = "0.0.0.0:502")]
    target: SocketAddr,

    /// How often to update persistence
    #[clap(short('f'), default_value = "1s", value_parser = validate_time)]
    update_frequency: Duration,

    /// Log Level (off, error, info, warn, trace)
    #[clap(short, default_value = "info", value_enum)]
    loglevel: log::LevelFilter,

    /// CIDR Whitelist (r/w/rw) (comma separated)
    #[clap(short = 'W', use_value_delimiter = true )]
    whitelist: Vec<String>,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let (read_whitelist, write_whitelist) = parse_whitelist(args.whitelist)?;
    
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
        .level(args.loglevel)
        .level_for("tokio_modbus", LevelFilter::Off)
        .chain(std::io::stdout())
        .apply().unwrap();

    println!("Starting with logging set to {}", args.loglevel);

    server::server_context(ServerConfig {
        socket_addr: args.target,
        update_frequency: args.update_frequency,
        read_whitelist,
        write_whitelist
    }).await?;

    Ok(())
}


#[cfg(test)]
mod test {
    use crate::parse_whitelist;

    type Error = Box<dyn std::error::Error>;


    #[test]
    pub fn test_parse_whitelist() -> Result<(), Error> {

        let strings = vec!["0.0.0.0/24:r".into(), "127.0.0.1".into(), "10.0.0.1/18:w".into()];
        let (read, write) = parse_whitelist(strings)?;

        assert!(read.as_ref().is_some_and(|r| r.iter().any(|r| r.contains("0.0.0.0".parse().unwrap()))));
        assert!(read.as_ref().is_some_and(|r| r.iter().any(|r| r.contains("0.0.0.155".parse().unwrap()))));
        assert!(read.as_ref().is_some_and(|r| r.iter().any(|r| !r.contains("0.0.10.155".parse().unwrap()))));
        assert!(write.as_ref().is_some_and(|r| r.iter().any(|r| r.contains("127.0.0.1".parse().unwrap()))));
        assert!(write.as_ref().is_some_and(|r| r.iter().any(|r| r.contains("10.0.0.25".parse().unwrap()))));


        assert!(parse_whitelist(vec![])?.0.is_none());
        assert!(parse_whitelist(vec![])?.1.is_none());

        Ok(())
    }

}
