use crate::register_manager::{RegisterError, RegisterManager, RegisterType};
use ipnetwork::IpNetwork;
use log::{debug, error, warn};
use std::{future, net::SocketAddr, sync::Arc};
use tokio_modbus::{ExceptionCode, Request, Response};

pub struct ModbusService {
    manager: Arc<RegisterManager>,
    read_whitelist: Option<Vec<IpNetwork>>,
    write_whitelist: Option<Vec<IpNetwork>>,
    ip: SocketAddr,
}

impl ModbusService {
    pub fn new(
        manager: Arc<RegisterManager>,
        ip: SocketAddr,
        read_whitelist: Option<Vec<IpNetwork>>,
        write_whitelist: Option<Vec<IpNetwork>>,
    ) -> Self {
        ModbusService {
            manager,
            read_whitelist,
            write_whitelist,
            ip,
        }
    }
}

impl From<RegisterError> for ExceptionCode {
    fn from(value: RegisterError) -> Self {
        match value {
            RegisterError::OutOfBounds => ExceptionCode::IllegalDataAddress,
            RegisterError::FileWriteError => ExceptionCode::ServerDeviceFailure,
        }
    }
}

impl tokio_modbus::server::Service for ModbusService {
    type Exception = tokio_modbus::ExceptionCode;
    type Response = tokio_modbus::Response;
    type Request = Request<'static>;
    type Future = future::Ready<Result<Response, ExceptionCode>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        if self
            .read_whitelist
            .as_ref()
            .is_some_and(|w| !w.iter().any(|ip| ip.contains(self.ip.ip())))
            && matches!(
                req,
                Request::WriteMultipleCoils(_, _)
                    | Request::WriteSingleCoil(_, _)
                    | Request::WriteMultipleRegisters(_, _)
                    | Request::WriteSingleRegister(_, _)
            )
        {
            warn!(
                "Blocked request {:?} from {}",
                req,
                self.ip.to_string()
            );
            return future::ready(Err(ExceptionCode::IllegalDataValue));
        }

        if self
            .write_whitelist
            .as_ref()
            .is_some_and(|w| !w.iter().any(|ip| ip.contains(self.ip.ip())))
            && matches!(
                req,
                Request::ReadCoils(_, _)
                    | Request::ReadDiscreteInputs(_, _)
                    | Request::ReadHoldingRegisters(_, _)
                    | Request::ReadInputRegisters(_, _)
            )
        {
            warn!(
                "Blocked request {:?} from {}",
                req,
                self.ip.to_string()
            );
            return future::ready(Err(ExceptionCode::IllegalDataValue));
        }

        debug!("{}: {:?}", self.ip, req);

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
                        Response::ReadDiscreteInputs(
                            reg.iter().map(|v| *v == 1).collect::<Vec<bool>>(),
                        )
                    })
                    .map_err(|e| e.into()),
            ),
            Request::ReadHoldingRegisters(addr, cnt) => future::ready(
                self.manager
                    .read_register(RegisterType::HoldingRegisters, addr, cnt)
                    .map(Response::ReadHoldingRegisters)
                    .map_err(|e| e.into()),
            ),
            Request::WriteMultipleRegisters(addr, values) => future::ready(
                self.manager
                    .write_register(RegisterType::HoldingRegisters, addr, &values)
                    .map(|_| Response::WriteMultipleRegisters(addr, values.len() as u16))
                    .map_err(|e| e.into()),
            ),
            Request::WriteSingleRegister(addr, value) => future::ready(
                self.manager
                    .write_register(RegisterType::HoldingRegisters, addr, &[value])
                    .map(|_| Response::WriteSingleRegister(addr, 1))
                    .map_err(|e| e.into()),
            ),
            _ => {
                error!("SERVER: Exception::IllegalFunction - Unimplemented function code in request: {req:?}");
                future::ready(Err(ExceptionCode::IllegalFunction))
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::ModbusService;
    use crate::{
        register_manager::{RegisterManager, RegisterType},
        util::AsWords,
    };
    use serde_json::json;
    use std::sync::Arc;
    use tokio::test;
    use tokio_modbus::{server::Service, Request};
    type Error = Box<dyn std::error::Error>;

    #[test]
    pub async fn read_register_test() -> Result<(), Error> {
        let json = json!({
            "40007/Q": 42,
        });

        let register_manager = Arc::new(RegisterManager::from_json(json).unwrap());
        let service = ModbusService::new(
            register_manager.clone(),
            "0.0.0.0:503".parse().unwrap(),
            None,
            None,
        );

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
}
