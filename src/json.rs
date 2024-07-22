use std::{collections::HashMap, fs::{File, OpenOptions}, io::{ErrorKind, Read, Write}};

use serde_json::{Map, Number, Value};

use crate::register_manager::Register;

#[derive(Debug)]
pub enum JsonError {
    Incomplete(String),
    Invalid(String),
    NoFile,
    Io(std::io::Error),
    Other(String)
}

#[derive(Debug)]
pub struct JsonResult {
    pub coils: Register,
    pub inputs: Register,
    pub input_registers: Register,
    pub holding_registers: Register,
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonError::Invalid(msg) => f.write_str(msg),
            JsonError::Incomplete(msg) => f.write_str(msg),
            JsonError::Other(msg) => f.write_str(msg),
            JsonError::Io(err) => f.write_str(format!("{}", err.to_string()).as_str()),
            JsonError::NoFile => f.write_str("No file")
        }
    }
}

pub fn load(path: &str) -> Result<Value, JsonError> {
    let mut file = match File::open(path) {
        Ok(v) => v,
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                return Err(JsonError::NoFile)
            }
            return Err(JsonError::Other(e.to_string()))
        }
    };

    let mut content = String::new();
    file.read_to_string(&mut content).map_err(|e| JsonError::Other(e.to_string()))?;

    let data: Value = serde_json::from_str(&content.as_str()).map_err(|e| JsonError::Other(e.to_string()))?;

    Ok(data)
}


pub fn parse(data: Value) -> Result<JsonResult, JsonError> {

    let coil_addresses = [1u16, 100];
    let mut coils: Register = HashMap::with_capacity(coil_addresses.len());

    for addr in coil_addresses {
        let entry = data.get(&addr.to_string())
            .ok_or(JsonError::Incomplete(format!("Missing address {}", addr)))?;
        let value = entry.as_bool()
            .ok_or(JsonError::Invalid(format!("Address {} should be boolean", addr)))?;

        coils.insert(addr, u16::from(value));
    }

    let h_u64 = [
        40001u16,
        40002,
        40003,
        40004,
        40005,
        40006,
        40007,
        40008,
        40100
    ];
    let mut holding_registers: Register = HashMap::with_capacity(h_u64.len());

    for addr in h_u64 {
        let entry = data.get(&addr.to_string())
            .ok_or(JsonError::Incomplete(format!("Missing address {}", addr)))?;
        let value = entry.as_u64()
            .and_then(|v| u16::try_from(v).ok()).ok_or(JsonError::Invalid(format!("Address {addr} must be u16")))?;

        holding_registers.insert(addr, value);
    }

    let h_i64 = [40200u16];

    for addr in h_i64 {
        let entry = data.get(&addr.to_string())
            .ok_or(JsonError::Incomplete(format!("Missing address {}", addr)))?;
        let value = entry.as_i64()
            .and_then(|v| i16::try_from(v).ok()).ok_or(JsonError::Invalid(format!("Address {addr} must be i16")))?;

        holding_registers.insert(addr, value as u16);
    }

    let inputs: HashMap<u16, u16> = HashMap::new();
    let input_registers: HashMap<u16, u16> = HashMap::new();
    
    Ok(JsonResult { coils, inputs, input_registers, holding_registers })
}

pub fn write(registers: &JsonResult, path: &str) -> Result<(), JsonError> {
    let JsonResult { coils, holding_registers, .. } = registers;
    let mut json: Map<String, Value> = Map::new();

    let b_u16 = [1u16, 100];
    for addr in b_u16 {
        let entry = coils.get(&addr)
            .ok_or(JsonError::Incomplete(format!("Missing address {}", addr)))?;
        let value = serde_json::Value::Bool(*entry == 1);

        json.insert(addr.to_string(), value);
    }


    
    let r_u16 = [
        40001, 
        40002,
        40003,
        40004,
        40005,
        40006,
        40007,
        40008,
        40100
    ];
    for addr in r_u16 {
        let entry = holding_registers.get(&addr)
            .ok_or(JsonError::Incomplete(format!("Missing address {}", addr)))?;
        let value = Value::Number(Number::from(*entry));

        json.insert(addr.to_string(), value);
    }

    let h_i16 = [40200u16];
    
    for addr in h_i16 {
        let entry = holding_registers.get(&addr).ok_or(JsonError::Incomplete(format!("Missing address {addr}")))?;
        let value = Value::Number(Number::from(*entry as i16));

        json.insert(addr.to_string(), value);
    }

    // let mut file
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(|e| JsonError::Io(e))?;

    let string = serde_json::to_string_pretty(&json).map_err(|_| JsonError::Other("Error converting to string".into()))?;
    
    file.write(string.as_bytes()).map_err(|e| JsonError::Io(e))?;

    file.flush().map_err(|e| JsonError::Io(e))?;


    Ok(())
}


#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::register_manager::RegisterManager;

    use super::*;
    type Error = Box<dyn std::error::Error>;

    #[test]
    pub fn test_parse_data() -> Result<(), Error> {

        let data = json!({
            "1": false,
            "100": false,
            "40001": 0,
            "40002": 0,
            "40003": 0,
            "40004": 0,
            "40005": 0,
            "40006": 0,
            "40007": 0,
            "40008": 0,
            "40100": 0,
            "40200": 0,
            "40100": -10,
            "40200": 0,
        });

        let result = parse(data).map_err(|e| e.to_string());
        assert!(result.is_ok());

        let data = json!({
            "1": false,
            "100": false,
            "40001": 0,
            "40002": 0,
            "40003": 0,
            "40004": 0,
            "40005": 4000,
            "40006": 0,
            "40007": 0,
            "40008": 0,
            "40100": 0,
            "40200": 0,
            "40100": 0,
            "40200": 32768,
        });

        let result = parse(data).map_err(|e| e.to_string());
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    pub fn test_write_data() -> Result<(), Error> {
        let register = RegisterManager::new();
        let json = register.to_json();

        let _ = write(&json, "data.json");

        assert!(true);

        Ok(())
    }

}