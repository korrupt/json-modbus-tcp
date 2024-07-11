use std::{collections::HashMap, future, ops::Deref, sync::Arc};
use tokio_modbus::{Exception, Request, Response};
use crate::{json::{self, JsonResult}, register_manager::{RegisterError, RegisterManager, RegisterType}};

pub struct ModbusService {
    manager: Arc<RegisterManager>
}

impl ModbusService {
    pub fn new(manager: Arc<RegisterManager>) -> Self {
        ModbusService {
            manager
        }
    }
}

impl From<RegisterError> for Exception {
    fn from(value: RegisterError) -> Self {
        match value {
            RegisterError::OutOfBounds => Exception::IllegalDataAddress,
            RegisterError::FileWriteError => Exception::ServerDeviceFailure
        }
    }
}

impl tokio_modbus::server::Service for ModbusService {
    type Request = Request<'static>;
    type Future = future::Ready<Result<Response, Exception>>;
    
    fn call(&self, req: Self::Request) -> Self::Future {
        match req {
            Request::ReadCoils(addr, cnt) => future::ready(
                self.manager.read_register(RegisterType::Coils, addr, cnt)
                    .map(|reg| Response::ReadCoils(reg.iter().map(|v| *v == 1).collect::<Vec<bool>>()))
                    .map_err(|e| e.into())
            ),
            Request::WriteSingleCoil(addr, val) => future::ready(
                self.manager.write_register(RegisterType::Coils, addr, &[val as u16])
                    .map(|_|Response::WriteSingleCoil(addr, val))
                    .map_err(|e| e.into())
            ),
            Request::ReadInputRegisters(addr, cnt) => future::ready(
                self.manager.read_register(RegisterType::InputRegisters, addr, cnt)
                    .map(Response::ReadInputRegisters)
                    .map_err(|e| e.into())
            ),
            Request::ReadHoldingRegisters(addr, cnt) => future::ready(
                self.manager.read_register(RegisterType::HoldingRegisters, addr, cnt)
                    .map(Response::ReadHoldingRegisters)
                    .map_err(|e| e.into())
            ),
            Request::WriteMultipleRegisters(addr, values) => future::ready(
                self.manager.write_register(RegisterType::HoldingRegisters, addr, &values)
                    .map(|_| Response::WriteMultipleRegisters(addr, values.len() as u16))
                    .map_err(|e| e.into())
            ),
            Request::WriteSingleRegister(addr, value) => future::ready(
                self.manager.write_register(RegisterType::HoldingRegisters, addr, &[value])
                    .map(|_| Response::WriteMultipleRegisters(addr, 1))
                    .map_err(|e| e.into())
            ),
            _ => {
                println!("SERVER: Exception::IllegalFunction - Unimplemented function code in request: {req:?}");
                future::ready(Err(Exception::IllegalFunction))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::register_manager::create_empty_register;


    #[test]
    pub fn read_register_test() -> Result<(), anyhow::Error> {

        let register = create_empty_register([1..2, 100..104]);


        // assert_eq!(
        //     read_register(&register, 1, 1)?,
        //     vec![0u16]
        // );

        // assert_eq!(
        //     read_register(&register, 100, 4)?,
        //     vec![0u16, 0u16, 0u16, 0u16]
        // );


        Ok(())
    }

}