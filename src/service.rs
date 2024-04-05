use std::{
    collections::HashMap,
    future,
    sync::{Arc, Mutex},
};

use tokio_modbus::prelude::*;


pub struct BatteryService {
    #[allow(unused)]
    inputs: Arc<Mutex<HashMap<u16, u16>>>,
    coils: Arc<Mutex<HashMap<u16, u16>>>,
    input_registers: Arc<Mutex<HashMap<u16, u16>>>,
    holding_registers: Arc<Mutex<HashMap<u16, u16>>>,
}



impl BatteryService {
    pub fn new() -> Self {
        // Insert some test data as register values.
        let inputs = HashMap::new();
        let mut coils = HashMap::new();

        coils.insert(1, 0);
        coils.insert(100, 0);

        let input_registers = HashMap::new();
        let mut holding_registers = HashMap::new();

        let ranges: Vec<u16> = vec![
            (1..22),
            (100..100),
            (200..200)
        ].into_iter()
        .flat_map(|r| r.into_iter().collect::<Vec<u16>>())
        .collect();

        for addr in ranges {
            holding_registers.insert(addr, 0);
        }

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
                    .map(|reg| Response::ReadCoils(reg.iter().map(|v| *v == 1).collect::<Vec<bool>>()))
            ),
            Request::WriteSingleCoil(addr, val) => future::ready(
                register_write(&mut self.coils.lock().unwrap(), addr, &[val as u16])
                    .map(|_|Response::WriteSingleCoil(addr, val))
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


#[cfg(test)]
mod async_tests {
    use tokio_modbus::server::Service;

    use crate::util::AsWords;

    use super::*;

    type Error = Box<dyn std::error::Error>;
    
    #[tokio::test]
    pub async fn test_coil() -> Result<(), Error> {
        let service = BatteryService::new();

        let initial = service.call(Request::ReadCoils(1, 1)).await?;

        assert_eq!(initial, Response::ReadCoils(vec![false]));

        let updated = service.call(Request::WriteSingleCoil(1, true)).await?;

        assert_eq!(updated, Response::WriteSingleCoil(1, true));

        let illegal = service.call(Request::WriteSingleCoil(13, false)).await;

        assert_eq!(illegal, Err(Exception::IllegalDataAddress));

        Ok(())
    }

    #[tokio::test]
    pub async fn test_holding_register() -> Result<(), Error> {
        let service = BatteryService::new();

        let initial = service.call(Request::ReadHoldingRegisters(1, 6)).await?;

        assert_eq!(initial, Response::ReadHoldingRegisters(vec![0, 0, 0, 0, 0, 0]));

        // updating values
        let updated_values = vec![
            vec![
                0x0001,
                0x0000,
            ],
            (-42i64).as_words()
        ].concat();

        service.call(Request::WriteMultipleRegisters(1, updated_values.into())).await?;

        let tot_power = service.call(Request::ReadHoldingRegisters(3, 4)).await?;

        assert_eq!(tot_power, Response::ReadHoldingRegisters((-42i64).as_words()));

        Ok(())
    }

}