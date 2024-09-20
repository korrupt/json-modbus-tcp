use std::{net::{Ipv4Addr, SocketAddr}, str::FromStr, time::Duration};

use clap::Parser;

mod service;
mod server;
mod util;
mod json;
mod register_manager;



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
    debug: bool
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
    
    Err(String::from("The time must be a whole number suffixed by 's', 'ms', or 'us'"))
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Args::parse();

    let ip_address = Ipv4Addr::from_str(&args.target).expect("Invalid IP address:");
    let socket_addr: SocketAddr = (ip_address, args.port).into();
    let debug = args.debug;

    if debug {
        println!("STARTING IN DEBUG MODE");
    }

    server::server_context(socket_addr, args.update_frequency, debug).await?;
    
    Ok(())
}
