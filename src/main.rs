use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
    time::Duration,
};

use clap::Parser;
use ipnetwork::IpNetwork;
use server::ServerConfig;

mod json;
mod python_struct;
mod register_manager;
mod server;
mod service;
mod util;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// target IP address
    target: String,

    /// Which port to run on
    #[clap(short, default_value = "503")]
    port: u16,

    /// How often to update persistence
    #[clap(short('f'), default_value = "1s", value_parser = validate_time)]
    update_frequency: Duration,

    /// Debug mode
    #[clap(long)]
    debug: bool,

    /// Whitelist
    #[clap(short = 'W', use_value_delimiter = true )]
    whitelist: Vec<String>,
}

pub enum Op {
    Read,
    Write,
    ReadWrite,
}

impl Op {
    pub fn parse(target: &str) -> Result<Self, String> {
        match target {
            "r" => Ok(Self::Read),
            "w" => Ok(Self::Write),
            "rw" => Ok(Self::ReadWrite),
            _ => Err("Error parsing operation".into()),
        }
    }
}

pub fn parse_whitelist(
    target: Vec<String>,
) -> Result<(Option<Vec<IpNetwork>>, Option<Vec<IpNetwork>>), String> {
    let mut read_whitelist: Vec<IpNetwork> = Vec::new();
    let mut write_whitelist: Vec<IpNetwork> = Vec::new();

    for cidr_string in &target {
        let (net, op) = cidr_string
        .find(":")
        .map_or(Ok((cidr_string.as_str(), Op::ReadWrite)), |idx| {
            Op::parse(&cidr_string[(idx + 1)..]).map(|op| (&cidr_string[..idx], op))
        })
        .and_then(|(c, op)| {
            c.parse::<IpNetwork>()
                .map(|network| (network, op))
                .map_err(|e| format!("Error parsing CIDR part of string: {}", e))
        })?;

        if matches!(op, Op::Read | Op::ReadWrite) {
            read_whitelist.push(net);
        }

        if matches!(op, Op::Write | Op::ReadWrite) {
            write_whitelist.push(net);
        }
    }

    Ok((
        Some(read_whitelist).filter(|w| w.len() > 0),
        Some(write_whitelist).filter(|w| w.len() > 0),
    ))
    // Ok((read_whitelist, write_whitelist))
}

fn validate_time(val: &str) -> Result<Duration, String> {
    if let Some(suffix) = val.strip_suffix("ms") {
        if let Ok(num) = suffix.parse::<u64>() {
            return Ok(Duration::from_millis(num));
        }
    } else if let Some(suffix) = val.strip_suffix("us") {
        if let Ok(num) = suffix.parse::<u64>() {
            return Ok(Duration::from_micros(num));
        }
    } else if let Some(suffix) = val.strip_suffix("s") {
        if let Ok(num) = suffix.parse::<u64>() {
            return Ok(Duration::from_secs(num));
        }
    }

    Err(String::from(
        "The time must be a whole number suffixed by 's', 'ms', or 'us'",
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let ip_address = Ipv4Addr::from_str(&args.target).expect("Invalid IP address:");
    let socket_addr: SocketAddr = (ip_address, args.port).into();
    let debug = args.debug;

    let (read_whitelist, write_whitelist) = parse_whitelist(args.whitelist)?;

    if debug {
        println!("STARTING IN DEBUG MODE");
    }

    server::server_context(ServerConfig {
        socket_addr,
        debug,
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

        Ok(())
    }

}
