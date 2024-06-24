use std::{
    collections::HashMap, fs::File, future, io::{BufWriter, Seek, Write}, str::FromStr, sync::{Arc, Mutex}
};

use serde_json::{Number, Value};
use tokio_modbus::prelude::*;

use crate::json::{self, JsonError, JsonResult};
use crate::util::FromVec;


pub struct BatteryService {
    #[allow(unused)]
    inputs: Arc<Mutex<HashMap<u16, u16>>>,
    coils: Arc<Mutex<HashMap<u16, u16>>>,
    input_registers: Arc<Mutex<HashMap<u16, u16>>>,
    holding_registers: Arc<Mutex<HashMap<u16, u16>>>,
}



impl BatteryService {

    pub fn write_json(&self) -> Result<(), JsonError> {
        let mut json = json::load_json("data.json").unwrap();

        match &mut json {
            Value::Object(m) => {
                let holding_registers_copy = self.holding_registers.lock().unwrap().clone();

                let h_u16 = [
                    40001u16,
                    40002
                ];

                for k in h_u16 {
                    let val = holding_registers_copy.get(&k).unwrap();
                    let number = Number::from_str(val.to_string().as_str()).unwrap();
                    m.insert(k.to_string(), Value::Number(number));
                }



                let h_u64 = [
                    40003u16,
                    40007,
                    40011,
                    40015,
                    40019,
                    40023,
                    40100,
                ];

                for k in h_u64 {
                    let val = register_read(&holding_registers_copy, k, 4).unwrap();
                    let transformed: u64 = val.from_vec();
                    
                    let number = Number::from_str(transformed.to_string().as_str()).unwrap();
                    // println!("Val: {} transformed {} Number {}", k.to_string(), transformed.to_string(), number);

                    m.insert(k.to_string(), Value::Number(number));
                }

                let h_i64 = [
                    40200u16
                ];


                for k in h_i64 {
                    // let val = holding_registers_copy.get(k)
                    let val = register_read(&holding_registers_copy, k, 4).unwrap();
                    let transformed: i64 = val.from_vec();
                
                    let number = Number::from_str(transformed.to_string().as_str()).unwrap();

                    m.insert(k.to_string(), Value::Number(number));
                }

            },
            _ => panic!("WRONG FILE")
        };

        
        let mut file = File::options()
            .read(true)
            .write(true)
            .truncate(true)
            .open("data.json")
            .unwrap();

        
        let output = serde_json::to_string_pretty(&json).unwrap();
        file.write_all(output.as_bytes()).unwrap();


        Ok(())
    }

    pub fn try_from_json(data: Value) -> Result<BatteryService, JsonError> {
        let JsonResult { holding_registers, coils } = json::parse_data(data)?;

        Ok(BatteryService {
            holding_registers: Arc::new(Mutex::new(holding_registers)),
            coils: Arc::new(Mutex::new(coils)),
            input_registers: Arc::new(Mutex::new(HashMap::new())),
            inputs: Arc::new(Mutex::new(HashMap::new()))
        })
    }

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

        let op = match req {
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
        };
        
        self.write_json().unwrap();

        return op
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
    use serde_json::json;
    use tokio_modbus::server::Service;

    use crate::util::AsWords;

    use super::*;

    type Error = Box<dyn std::error::Error>;

    #[tokio::test]
    pub async fn load_from_json() -> Result<(), Error> {
        let data = json!({
            "00001": false,
            "00100": false,
            "40001": 1,
            "40002": 1,
            "40003": 20000,
            "40007": 69696969,
            "40011": 69696969,
            "40015": 69696969,
            "40019": 69696969,
            "40023": 69696969,
            "40100": 69696969,
            "40200": -69696969,
        });

        let service = BatteryService::try_from_json(data).expect("Error loading json");

        let coil = service.call(Request::ReadCoils(1, 1)).await?;

        assert_eq!(coil, Response::ReadCoils(vec![false]));

        let holding_register = service.call(Request::ReadHoldingRegisters(40007, 8)).await?;

        assert_eq!(holding_register, Response::ReadHoldingRegisters([
            69696969u64.as_words(),
            69696969u64.as_words(),
        ].concat()));

        Ok(())
    }
    
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
