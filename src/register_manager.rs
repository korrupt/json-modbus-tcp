use std::{collections::HashMap, sync::{Arc, RwLock}};

use serde_json::Value;

use crate::json::{self, JsonError, JsonResult};

pub type Register = HashMap<u16, u16>;

#[derive(Debug)]
pub enum RegisterError {
    OutOfBounds,
    FileWriteError,
}

pub fn create_empty_register<T, const M: usize>(ranges: [T; M]) -> Register
    where T: IntoIterator<Item = u16> {
        ranges.into_iter()
            .flat_map(|e| e.into_iter().collect::<Vec<u16>>())
            .map(|e| (e, 0))
            .collect::<Register>()
}

impl Default for RegisterManager {
    fn default() -> Self {
        let inputs  = create_empty_register([[0]]);
        let coils   = create_empty_register([[1, 100]]);
        let holding_registers = create_empty_register([40001..40027, 40100..40104, 40200..40204]);
        let input_registers = create_empty_register([0..0]); 

        RegisterManager {
            inputs: Arc::new(RwLock::new(inputs)),
            coils: Arc::new(RwLock::new(coils)),
            holding_registers: Arc::new(RwLock::new(holding_registers)),
            input_registers: Arc::new(RwLock::new(input_registers)),
        }
    }
}

pub struct RegisterManager {
    inputs: Arc<RwLock<Register>>,   
    coils: Arc<RwLock<Register>>,   
    holding_registers: Arc<RwLock<Register>>,   
    input_registers: Arc<RwLock<Register>>,
}

#[allow(dead_code)]
pub enum RegisterType {
    Inputs,
    Coils,
    HoldingRegisters,
    InputRegisters
}

impl RegisterManager {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_json(json: Value) -> Result<Self, JsonError> {
        let JsonResult { coils, holding_registers, .. } = json::parse(json)?;

        Ok(RegisterManager {
            coils: Arc::new(RwLock::new(coils)),
            holding_registers: Arc::new(RwLock::new(holding_registers)),
            ..Default::default()
        })
    }

    pub fn to_json(&self) -> JsonResult {
        JsonResult {
            coils: self.coils.read().unwrap().clone(),
            inputs: self.inputs.read().unwrap().clone(),
            input_registers: self.input_registers.read().unwrap().clone(),
            holding_registers: self.holding_registers.read().unwrap().clone(),
        }
    }

    pub fn update_persistence(&self) -> Result<(), RegisterError> {
        println!("Updating persistence");

        let result = self.to_json();

        json::write(&result, "data.json").unwrap();

        Ok(())
    }

    fn register_select(&self, registers_type: RegisterType) -> &Arc<RwLock<Register>> {
        match registers_type {
            RegisterType::Coils => &self.coils,
            RegisterType::HoldingRegisters => &self.holding_registers,
            RegisterType::InputRegisters => &self.input_registers,
            RegisterType::Inputs => &self.input_registers
        }
    }

    pub fn read_register(
        &self,
        registers_type: RegisterType,
        addr: u16,
        cnt: u16
    ) -> Result<Vec<u16>, RegisterError> {
        let mut response: Vec<u16> = Vec::with_capacity(cnt.into());

        let registers = self.register_select(registers_type)
            .read()
            .unwrap();

        for i in 0..cnt {
            if let Some(value) = registers.get(&(addr + i)) {
                response.push(*value);
            } else {
                return Err(RegisterError::OutOfBounds);
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
        let mut registers = self.register_select(registers_type)
            .write()
            .unwrap();

        for (i, value) in values.iter().enumerate() {
            let reg_addr = addr + i as u16;
    
            if let Some(val) = registers.get_mut(&reg_addr) {
                *val = *value;
            } else {
                return Err(RegisterError::OutOfBounds);
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
            "1": false,
            "100": true,
            "40001": 1,
            "40002": 1,
            "40003": 20000,
            "40007": 69696969,
            "40011": 69696969,
            "40015": 69696969,
            "40019": 69696969,
            "40023": 69696969,
            "40100": 69696969,
            "40200": 69696969,
        });

        let _ = RegisterManager::from_json(data).unwrap();
        assert!(true);
        
        Ok(())
    }
}