use std::{collections::HashMap, fs::File, io::{ErrorKind, Read}};

use serde_json::Value;

use crate::util::AsWords;

#[derive(Debug)]
pub enum JsonError {
    Incomplete(String),
    Invalid(String),
    NoFile,
    Other(String)
}

pub struct JsonResult {
    pub coils: HashMap<u16, u16>,
    pub holding_registers: HashMap<u16, u16>,
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonError::Invalid(msg) => f.write_str(msg),
            JsonError::Incomplete(msg) => f.write_str(msg),
            JsonError::Other(msg) => f.write_str(msg),
            JsonError::NoFile => f.write_str("No file")
        }
    }
}

pub fn load_json(path: &str) -> Result<Value, JsonError> {
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

pub fn parse_data(data: Value) -> Result<JsonResult, JsonError> {
    
    let coils = [
        1u16,
        100,
    ];
        
    let mut coil_bytes: HashMap<u16, u16> = HashMap::with_capacity(coils.len());

    for addr in coils {
        // add 0-padding
        let modbus_addr = format!("{:05}", addr);
        
        let data = data.get(&modbus_addr).ok_or(JsonError::Incomplete(format!("Missing address: {}", &modbus_addr)))?;
        let value = data.as_bool().ok_or(JsonError::Invalid(format!("Expected bool at address {}", &modbus_addr)))?;

        coil_bytes.insert(addr, u16::from(value));
    }

    let holding_registers = [
        40001u16,
        40002,
        40003,
        40007,
        40011,
        40015,
        40019,
        40023,
        40100,
        40200
    ];


    let mut holding_register_bytes: HashMap<u16, u16> = HashMap::new();

    for addr in holding_registers {
        let data = data.get(addr.to_string()).ok_or(JsonError::Incomplete(format!("Missing address: {}", addr)))?;
        let value = data.as_i64().ok_or(JsonError::Invalid(format!("Expected u64 at address {}", addr)))?;

        for (i, slice) in value.as_words().iter().enumerate() {
            holding_register_bytes.insert(addr + (i as u16), *slice);
        }
    }

    Ok(JsonResult { coils: coil_bytes, holding_registers: holding_register_bytes })
}


#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    type Error = Box<dyn std::error::Error>;

    #[test]
    pub fn test_parse_data() -> Result<(), Error> {

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

        parse_data(data).map_err(|e| e.to_string())?;


        Ok(())
    }

}