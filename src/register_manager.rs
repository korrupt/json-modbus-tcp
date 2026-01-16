use std::{
    collections::HashMap, sync::{Arc, RwLock}
};

use log::warn;
use serde::{Serialize};
use thiserror::Error;
use crate::{register::{Canonical, Register, RegisterDefinition, RegisterError, RegisterType}};

#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("{0}")]
    Register(#[from] RegisterError),
    #[error("Out of bounds")]
    OutOfBounds,
}

pub struct RegisterManager {
    inputs: Arc<RwLock<Register>>,
    coils: Arc<RwLock<Register>>,
    holding_registers: Arc<RwLock<Register>>,
    input_registers: Arc<RwLock<Register>>,
}


impl RegisterManager {

    pub fn try_from_definitions(definitions: &[RegisterDefinition]) -> Result<Self, ManagerError> {
        let canonical = Canonical::try_from_definition(definitions)?;
        Ok(Self::from_canonical(canonical))
    }

    pub fn from_canonical(canonical: Canonical) -> Self {

        let mut coils: Register = HashMap::new();
        let mut inputs: Register = HashMap::new();
        let mut input_registers: Register = HashMap::new();
        let mut holding_registers: Register = HashMap::new();

        for (k, v) in canonical.to_register().into_iter() {
            match RegisterType::from_key(&k).expect("This has already been validated") {
                RegisterType::Coils => coils.insert(k, v),
                RegisterType::Inputs => inputs.insert(k, v),
                RegisterType::InputRegisters => input_registers.insert(k, v),
                RegisterType::HoldingRegisters => holding_registers.insert(k, v)
            };
        }

        
        RegisterManager {
            coils: Arc::new(RwLock::new(coils)),
            inputs: Arc::new(RwLock::new(inputs)),
            input_registers: Arc::new(RwLock::new(input_registers)),
            holding_registers: Arc::new(RwLock::new(holding_registers)),
        }
    }

    #[allow(dead_code)]
    pub fn new() -> Self {
        RegisterManager {
            ..Default::default()
        }
    }

    pub fn snapshot(&self) -> ManagerSnapshot {
        let coils = self.coils.read().unwrap().clone();
        let inputs = self.inputs.read().unwrap().clone();
        let input_registers = self.input_registers.read().unwrap().clone();
        let holding_registers = self.holding_registers.read().unwrap().clone();

        ManagerSnapshot { inputs, coils, holding_registers, input_registers }
    }

    fn register_select(&self, registers_type: RegisterType) -> &Arc<RwLock<Register>> {
        match registers_type {
            RegisterType::Coils => &self.coils,
            RegisterType::HoldingRegisters => &self.holding_registers,
            RegisterType::InputRegisters => &self.input_registers,
            RegisterType::Inputs => &self.inputs,
        }
    }

    pub fn read_register(
        &self,
        registers_type: RegisterType,
        addr: u16,
        cnt: u16,
    ) -> Result<Vec<u16>, ManagerError> {
        let registers = self.register_select(registers_type).read().unwrap();
        let response = read_register(&registers, addr, cnt)?;

        Ok(response)
    }

    pub fn write_register(
        &self,
        registers_type: RegisterType,
        addr: u16,
        values: &[u16],
    ) -> Result<(), ManagerError> {
        {
            let mut registers = self.register_select(registers_type).write().unwrap();

            write_register(&mut registers, addr, values)?;
        }

        Ok(())
    }
}


pub fn read_register(
    registers: &Register,
    addr: u16,
    cnt: u16,
) -> Result<Vec<u16>, ManagerError> {
    let mut response: Vec<u16> = Vec::with_capacity(cnt.into());

    for i in 0..cnt as usize {
        let reg_addr = addr + i as u16;
        if let Some(val) = registers.get(&reg_addr) {
            response.insert(i, *val);
        } else {
            warn!("Got register out of bounds at {}", &reg_addr);
            return Err(ManagerError::OutOfBounds);
        }
    }

    Ok(response)
}

pub fn write_register(
    registers: &mut Register,
    addr: u16,
    values: &[u16],
) -> Result<(), ManagerError> {
    for (i, value) in values.iter().enumerate() {
        let reg_addr = addr + i as u16;

        if let Some(val) = registers.get_mut(&reg_addr) {
            *val = *value;
        } else {
            warn!("Got register out of bounds at {}", &reg_addr);
            return Err(ManagerError::OutOfBounds);
        }
    }

    Ok(())
}


#[derive(Debug, Serialize)]
pub struct ManagerSnapshot {
    pub inputs: Register,
    pub coils: Register,
    pub holding_registers: Register,
    pub input_registers: Register,
}

impl Into<Register> for ManagerSnapshot {
    fn into(self) -> Register {
        let mut reg: Register = Register::new();

        for (k, v) in self.coils { reg.insert(k, v); };
        for (k, v) in self.inputs { reg.insert(k, v); };
        for (k, v) in self.input_registers { reg.insert(k, v); };
        for (k, v) in self.holding_registers { reg.insert(k, v); };

        reg
    }
}


impl Default for RegisterManager {
    fn default() -> Self {
        RegisterManager {
            inputs: Arc::new(RwLock::new(HashMap::new())),
            coils: Arc::new(RwLock::new(HashMap::new())),
            holding_registers: Arc::new(RwLock::new(HashMap::new())),
            input_registers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
