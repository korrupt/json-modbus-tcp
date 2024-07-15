use std::{future, sync::Arc};
use tokio_modbus::{Exception, Request, Response};
use crate::register_manager::{RegisterError, RegisterManager, RegisterType};

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

    use std::sync::Arc;
    use tokio::test;
    use tokio_modbus::{server::Service, Request};
    use crate::{register_manager::{RegisterManager, RegisterType}, util::AsWords};
    use super::ModbusService;

    #[test]
    pub async fn read_register_test() -> Result<(), anyhow::Error> {

        let register_manager = Arc::new(RegisterManager::new());
        let service = ModbusService::new(register_manager.clone());

        let value: u64 = 42;
        let value_arr = value.as_words();
        
        service.call(Request::WriteMultipleRegisters(40007, value_arr.clone().into())).await.unwrap();

        let received = service.manager.read_register(RegisterType::HoldingRegisters, 40007, 4).unwrap();

        assert_eq!(value_arr, received);

        Ok(())
    }

}