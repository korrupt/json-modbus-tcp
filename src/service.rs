use crate::{register::RegisterType, register_manager::{ManagerError, RegisterManager}, validation::Whitelist};
use log::{debug, error, warn};
use std::{future, net::SocketAddr, sync::Arc};
use tokio_modbus::{ExceptionCode, Request, Response};

pub struct ModbusService {
    manager: Arc<RegisterManager>,
    whitelist: Whitelist,
    ip: SocketAddr,
}

impl ModbusService {
    pub fn new(
        manager: Arc<RegisterManager>,
        ip: SocketAddr,
        whitelist: Whitelist,
    ) -> Self {
        ModbusService {
            manager,
            whitelist,
            ip,
        }
    }
}

impl From<ManagerError> for ExceptionCode {
    fn from(value: ManagerError) -> Self {
        match value {
            ManagerError::OutOfBounds => ExceptionCode::IllegalDataAddress,
            ManagerError::Register(_) => unreachable!()
        }
    }
}

impl tokio_modbus::server::Service for ModbusService {
    type Exception = tokio_modbus::ExceptionCode;
    type Response = tokio_modbus::Response;
    type Request = Request<'static>;
    type Future = future::Ready<Result<Response, ExceptionCode>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        debug!("Got request {:?}", req);
        if !self
            .whitelist
            .read
            .iter().any(|ip| ip.contains(self.ip.ip()))
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

        if !self
            .whitelist
            .write
            .iter().any(|ip| ip.contains(self.ip.ip()))
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
