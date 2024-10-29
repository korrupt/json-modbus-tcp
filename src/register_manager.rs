use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use serde_json::Value;

use crate::json::{self, JsonError};

pub type Register = HashMap<u16, u16>;

#[derive(Debug)]
pub enum RegisterError {
    OutOfBounds,
    FileWriteError,
}

impl Default for RegisterManager {
    fn default() -> Self {
        RegisterManager {
            inputs: Arc::new(RwLock::new(HashMap::new())),
            coils: Arc::new(RwLock::new(HashMap::new())),
            holding_registers: Arc::new(RwLock::new(HashMap::new())),
            input_registers: Arc::new(RwLock::new(HashMap::new())),
            keys: vec![],
        }
    }
}

pub struct RegisterManager {
    inputs: Arc<RwLock<Register>>,
    coils: Arc<RwLock<Register>>,
    holding_registers: Arc<RwLock<Register>>,
    input_registers: Arc<RwLock<Register>>,
    keys: Vec<String>,
}

#[allow(dead_code)]
pub enum RegisterType {
    Inputs,
    Coils,
    HoldingRegisters,
    InputRegisters,
}

impl std::fmt::Display for RegisterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterType::Coils => f.write_str("Coils"),
            RegisterType::HoldingRegisters => f.write_str("Holding Registers"),
            RegisterType::InputRegisters => f.write_str("Input Registers"),
            RegisterType::Inputs => f.write_str("Inputs"),
        }
    }
}

impl RegisterManager {
    pub fn new() -> Self {
        RegisterManager {
            // debug,
            ..Default::default()
        }
    }

    pub fn from_json(json: Value) -> Result<Self, JsonError> {
        // let JsonResult { coils, holding_registers, .. } = json::parse(json)?;
        let (registers, keys) = json::parse(json)?;

        let coils = registers
            .keys()
            .into_iter()
            .cloned()
            .filter(|&key| key >= 1 && key <= 9999)
            .filter_map(|key| registers.get(&key).and_then(|val| Some((key, val.clone()))))
            .collect();

        let inputs = registers
            .keys()
            .into_iter()
            .cloned()
            .filter(|&key| key >= 10001 && key <= 19999)
            .filter_map(|key| registers.get(&key).and_then(|val| Some((key, val.clone()))))
            .collect();

        let input_registers = registers
            .keys()
            .into_iter()
            .cloned()
            .filter(|&key| key >= 30001 && key <= 39999)
            .filter_map(|key| registers.get(&key).and_then(|val| Some((key, val.clone()))))
            .collect();

        let holding_registers = registers
            .keys()
            .into_iter()
            .cloned()
            .filter(|&key| key >= 40001 && key <= 49999)
            .filter_map(|key| registers.get(&key).and_then(|val| Some((key, val.clone()))))
            .collect();

        Ok(RegisterManager {
            coils: Arc::new(RwLock::new(coils)),
            inputs: Arc::new(RwLock::new(inputs)),
            input_registers: Arc::new(RwLock::new(input_registers)),
            holding_registers: Arc::new(RwLock::new(holding_registers)),
            // debug,
            keys,
        })
    }

    pub fn update_persistence(&self) -> Result<(), RegisterError> {
        // if self.debug {
        //     println!("Updating persistence");
        // }

        let coils = self.coils.read().unwrap().clone();
        let inputs = self.inputs.read().unwrap().clone();
        let input_registers = self.input_registers.read().unwrap().clone();
        let holding_registers = self.holding_registers.read().unwrap().clone();

        let registers: HashMap<u16, u16> = coils
            .into_iter()
            .chain(inputs.into_iter())
            .chain(input_registers.into_iter())
            .chain(holding_registers.into_iter())
            .collect();

        let value = json::registers_to_object(&registers, self.keys.clone()).unwrap();

        if let Err(e) = json::write(value, "data.json") {
            eprintln!("Error updating persistence: {:?}", e);
        }

        Ok(())
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
    ) -> Result<Vec<u16>, RegisterError> {

        let addr = match registers_type {
            RegisterType::Coils => Some(addr).filter(|&a| a >= 1 && a <= 9999),
            RegisterType::Inputs => Some(addr).filter(|&a| a >= 10001 && a <= 19999),
            RegisterType::InputRegisters => Some(addr).filter(|&a| a >= 30001 && a <= 39999),
            RegisterType::HoldingRegisters => Some(addr).filter(|&a| a >= 40001 && a <= 49999),
        }.ok_or(RegisterError::OutOfBounds)?;
        
        let mut response: Vec<u16> = Vec::with_capacity(cnt.into());

        // if self.debug {
        //     println!("Read {} addr: {} Cnt: {:?}", registers_type, addr, cnt);
        // }

        {
            let registers = self.register_select(registers_type).read().unwrap();

            for i in 0..cnt {
                if let Some(value) = registers.get(&(addr + i)) {
                    response.push(*value);
                } else {
                    return Err(RegisterError::OutOfBounds);
                }
            }
        }

        Ok(response)
    }

    pub fn write_register(
        &self,
        registers_type: RegisterType,
        addr: u16,
        values: &[u16],
    ) -> Result<(), RegisterError> {
        {
            // if self.debug {
            //     println!("Write {} addr: {} Data: {:?}", registers_type, addr, values);
            // }

            let mut registers = self.register_select(registers_type).write().unwrap();

            for (i, value) in values.iter().enumerate() {
                let reg_addr = addr + i as u16;

                if let Some(val) = registers.get_mut(&reg_addr) {
                    *val = *value;
                } else {
                    return Err(RegisterError::OutOfBounds);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod register_tests {
    use serde_json::json;

    use crate::register_manager::RegisterManager;

    #[test]
    pub fn test_from_json() -> anyhow::Result<()> {
        let data = json!({
            "1": 1,
            "100": 1,
            "40001": 1,
            "40002": 1,
            "40003": 20000,
            "40007": 10,
            "40011": 10,
            "40015": 10,
            "40019": 10,
            "40023": 10,
            "40100": 10,
            "40200": 10,
        });

        let _ = RegisterManager::from_json(data).unwrap();
        assert!(true);

        Ok(())
    }
}
