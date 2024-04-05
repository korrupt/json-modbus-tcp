use std::{
    collections::HashMap,
    future,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio_modbus::{
    prelude::*,
    server::tcp::{accept_tcp_connection, Server},
};


pub struct BatteryService {
    inputs: Arc<Mutex<HashMap<u16, u16>>>,
    coils: Arc<Mutex<HashMap<u16, u16>>>,
    input_registers: Arc<Mutex<HashMap<u16, u16>>>,
    holding_registers: Arc<Mutex<HashMap<u16, u16>>>,
}



impl BatteryService {
    pub fn new() -> Self {
        // Insert some test data as register values.
        let inputs = HashMap::new();
        let coils = HashMap::new();
        let input_registers = HashMap::new();
        let holding_registers = HashMap::new();

        Self {
            inputs: Arc::new(Mutex::new(inputs)),
            coils: Arc::new(Mutex::new(coils)),
            input_registers: Arc::new(Mutex::new(input_registers)),
            holding_registers: Arc::new(Mutex::new(holding_registers)),
        }
    }
}

impl tokio_modbus::server::Service for BatteryService {
    type Request = Request<'static>;
    type Future = future::Ready<Result<Response, Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        match req {
            Request::ReadCoils(addr, cnt) => future::ready(
                register_read(&self.coils.lock().unwrap(), addr, cnt)
                    .map(|reg| Response::ReadCoils(reg.iter().map(|v| *v == 0).collect::<Vec<bool>>()))
            ),
            Request::ReadInputRegisters(addr, cnt) => future::ready(
                register_read(&self.input_registers.lock().unwrap(), addr, cnt)
                    .map(Response::ReadInputRegisters),
            ),
            Request::ReadHoldingRegisters(addr, cnt) => future::ready(
                register_read(&self.holding_registers.lock().unwrap(), addr, cnt)
                    .map(Response::ReadHoldingRegisters),
            ),
            Request::WriteMultipleRegisters(addr, values) => future::ready(
                register_write(&mut self.holding_registers.lock().unwrap(), addr, &values)
                    .map(|_| Response::WriteMultipleRegisters(addr, values.len() as u16)),
            ),
            Request::WriteSingleRegister(addr, value) => future::ready(
                register_write(
                    &mut self.holding_registers.lock().unwrap(),
                    addr,
                    std::slice::from_ref(&value),
                )
                .map(|_| Response::WriteSingleRegister(addr, value)),
            ),
            _ => {
                println!("SERVER: Exception::IllegalFunction - Unimplemented function code in request: {req:?}");
                future::ready(Err(Exception::IllegalFunction))
            }
        }
    }
}

fn register_read(
    registers: &HashMap<u16, u16>,
    addr: u16,
    cnt: u16,
) -> Result<Vec<u16>, Exception>
{
    let mut response_values: Vec<u16> = vec![0; cnt.into()];
    for i in 0..cnt {
        let reg_addr = addr + i;
        if let Some(r) = registers.get(&reg_addr) {
            response_values[i as usize] = *r;
        } else {
            println!("SERVER: Exception::IllegalDataAddress");
            return Err(Exception::IllegalDataAddress);
        }
    }

    Ok(response_values)
}

/// Write a holding register. Used by both the write single register
/// and write multiple registers requests.
fn register_write(
    registers: &mut HashMap<u16, u16>,
    addr: u16,
    values: &[u16],
) -> Result<(), Exception> {
    for (i, value) in values.iter().enumerate() {
        let reg_addr = addr + i as u16;
        if let Some(r) = registers.get_mut(&reg_addr) {
            *r = *value;
        } else {
            println!("SERVER: Exception::IllegalDataAddress");
            return Err(Exception::IllegalDataAddress);
        }
    }

    Ok(())
}