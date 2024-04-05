use std::net::{Ipv4Addr, SocketAddr};

use clap::Parser;

mod service;
mod server;
mod util;


// fn transform_coil_vector(coils: &Vec<u16>) -> Vec<u16> {
//     let num_bytes = (coils.len() + 15) / 16;
//     let mut packed_coils = vec![0u16; num_bytes];

//     for (index, &status) in coils.iter( ).enumerate() {
//         let u16_idx = index / 16;
//         let bit_position = index & 16;

//         if status != 0 {
//             packed_coils[u16_idx] |= 1 << bit_position;
//         }
//     }

//     packed_coils
// }


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Which port to run on
    port: u16,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Args::parse();

    let socket_addr: SocketAddr = (Ipv4Addr::UNSPECIFIED, args.port).into();

    server::server_context(socket_addr).await?;

    Ok(())
}

// async fn client_context(socket_addr: SocketAddr) {
//     tokio::join!(
//         async {
//             // Give the server some time for starting up
//             tokio::time::sleep(Duration::from_secs(1)).await;

//             println!("CLIENT: Connecting client...");
//             let mut ctx = tcp::connect(socket_addr).await.unwrap();

//             println!("CLIENT: Reading 2 input registers...");
//             let response = ctx.read_input_registers(0x00, 2).await.unwrap();
//             println!("CLIENT: The result is '{response:?}'");
//             assert_eq!(response, vec![1234, 5678]);

//             println!("CLIENT: Writing 2 holding registers...");
//             ctx.write_multiple_registers(0x01, &[7777, 8888])
//                 .await
//                 .unwrap();
//                 // .unwrap();

//             // Read back a block including the two registers we wrote.
//             println!("CLIENT: Reading 4 holding registers...");
//             let response = ctx.read_holding_registers(0x00, 4).await.unwrap();
//             println!("CLIENT: The result is '{response:?}'");
//             assert_eq!(response, vec![10, 7777, 8888, 40]);

//             // Now we try to read with an invalid register address.
//             // This should return a Modbus exception response with the code
//             // IllegalDataAddress.
//             // println!("CLIENT: Reading nonexistent holding register address... (should return IllegalDataAddress)");
//             // let response = ctx.read_holding_registers(0x100, 1).await;
//             // println!("CLIENT: The result is '{response:?}'");
//             // assert!(matches!(response, Err(Exception::IllegalDataAddress)));

//             println!("CLIENT: Done.")
//         },
//         tokio::time::sleep(Duration::from_secs(5))
//     );

//     let a = 1;
// }