use crate::register_manager::{RegisterError, RegisterManager, RegisterType};
use std::{future, sync::Arc};
use tokio_modbus::{Exception, Request, Response};

pub struct ModbusService {
    manager: Arc<RegisterManager>,
}

impl ModbusService {
    pub fn new(manager: Arc<RegisterManager>) -> Self {
        ModbusService { manager }
    }
}

impl From<RegisterError> for Exception {
    fn from(value: RegisterError) -> Self {
        match value {
            RegisterError::OutOfBounds => Exception::IllegalDataAddress,
            RegisterError::FileWriteError => Exception::ServerDeviceFailure,
        }
    }
}

impl tokio_modbus::server::Service for ModbusService {
    type Request = Request<'static>;
    type Future = future::Ready<Result<Response, Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        println!("Request: {:?}", req);
        match req {
            Request::ReadCoils(addr, cnt) => future::ready(
                self.manager
                    .read_register(RegisterType::Coils, addr, cnt)
                    .map(|reg| {
                        Response::ReadCoils(reg.iter().map(|v| *v == 1).collect::<Vec<bool>>())
                    })
                    .map_err(|e| e.into()),
            ),
            Request::WriteSingleCoil(addr, val) => future::ready(
                self.manager
                    .write_register(RegisterType::Coils, addr, &[val as u16])
                    .map(|_| Response::WriteSingleCoil(addr, val))
                    .map_err(|e| e.into()),
            ),
            Request::ReadInputRegisters(addr, cnt) => future::ready(
                self.manager
                    .read_register(RegisterType::InputRegisters, addr, cnt)
                    .map(Response::ReadInputRegisters)
                    .map_err(|e| e.into()),
            ),
            Request::ReadDiscreteInputs(addr, cnt) => future::ready(
                self.manager
                    .read_register(RegisterType::Inputs, addr, cnt)
                    .map(|reg| {
                        Response::ReadDiscreteInputs(reg.iter().map(|v| *v == 1).collect::<Vec<bool>>())
                    })
                    .map_err(|e| e.into()),
            ),
            Request::ReadHoldingRegisters(addr, cnt) => future::ready(
                self.manager
                    .read_register(
                        RegisterType::HoldingRegisters,
                        addr,
                        cnt,
                    )
                    .map(Response::ReadHoldingRegisters)
                    .map_err(|e| e.into())
            ),
            Request::WriteMultipleRegisters(addr, values) => future::ready(
                self.manager
                    .write_register(
                        RegisterType::HoldingRegisters,
                        addr,
                        &values,
                    )
                    .map(|_| Response::WriteMultipleRegisters(addr, values.len() as u16))
                    .map_err(|e| e.into()),
            ),
            Request::WriteSingleRegister(addr, value) => future::ready(
                self.manager
                    .write_register(
                        RegisterType::HoldingRegisters,
                        addr,
                        &[value],
                    )
                    .map(|_| Response::WriteSingleRegister(addr, 1))
                    .map_err(|e| e.into()),
            ),
            _ => {
                println!("SERVER: Exception::IllegalFunction - Unimplemented function code in request: {req:?}");
                future::ready(Err(Exception::IllegalFunction))
            }
        }
    }
}

fn pad_holding_register(addr: u16) -> u16 {
    if addr < 10000 {
        addr + 40000
    } else {
        addr
    }
}

#[cfg(test)]
mod tests {

    use super::ModbusService;
    use crate::{
        register_manager::{RegisterManager, RegisterType},
        util::AsWords,
    };
    use std::sync::Arc;
    use serde_json::json;
    use tokio::test;
    use tokio_modbus::{server::Service, Request};

    #[test]
    pub async fn read_register_test() -> Result<(), anyhow::Error> {

        let json = json!({
            "40007/Q": 42, 
        });

        let register_manager = Arc::new(RegisterManager::from_json(json, false).unwrap());
        let service = ModbusService::new(register_manager.clone());

        let value: u64 = 42;
        let value_arr = value.as_words();

        service
            .call(Request::WriteMultipleRegisters(
                40007,
                value_arr.clone().into(),
            ))
            .await
            .unwrap();

        let received = service
            .manager
            .read_register(RegisterType::HoldingRegisters, 40007, 4)
            .unwrap();

        assert_eq!(value_arr, received);

        Ok(())
    }

    #[test]
    pub async fn pad_address_test() -> Result<(), anyhow::Error> {
        Ok(())
    }
}

