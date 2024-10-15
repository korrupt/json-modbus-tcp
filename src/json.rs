use crate::pack::{PackFormat, PackType};
use serde_json::{Map, Value};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{ErrorKind, Read, Write},
    str::FromStr,
};

#[derive(Debug)]
pub enum JsonError {
    Incomplete(String),
    Invalid(String),
    NoFile,
    Io(std::io::Error),
    Other(String),
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonError::Invalid(msg) => f.write_str(msg),
            JsonError::Incomplete(msg) => f.write_str(msg),
            JsonError::Other(msg) => f.write_str(msg),
            JsonError::Io(err) => f.write_str(format!("{}", err.to_string()).as_str()),
            JsonError::NoFile => f.write_str("No file"),
        }
    }
}

pub fn load(path: &str) -> Result<Value, JsonError> {
    let mut file = match File::open(path) {
        Ok(v) => v,
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                return Err(JsonError::NoFile);
            }
            return Err(JsonError::Other(e.to_string()));
        }
    };

    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| JsonError::Other(e.to_string()))?;

    let data: Value =
        serde_json::from_str(&content.as_str()).map_err(|e| JsonError::Other(e.to_string()))?;

    Ok(data)
}

pub fn write(value: serde_json::Value, path: &str) -> Result<(), JsonError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(|e| JsonError::Io(e))?;

    let string = serde_json::to_string_pretty(&value)
        .map_err(|_| JsonError::Other("Error converting to string".into()))?;

    file.write(string.as_bytes())
        .map_err(|e| JsonError::Io(e))?;

    file.flush().map_err(|e| JsonError::Io(e))?;

    Ok(())
}

pub fn parse(data: Value) -> Result<(HashMap<u16, u16>, Vec<String>), JsonError> {
    if let Value::Object(ref map) = data {
        let keys: Vec<String> = map.keys().cloned().collect();
        let mut registers: HashMap<u16, u16> = HashMap::new(); // #TODO! measure length in advance

        for (k, v) in map {
            let PackFormat { address, pack_type } = PackFormat::parse(k)
                .map_err(|_| JsonError::Invalid(format!("Error parsing key '{}'", k)))?;

            let number = match v {
                Value::Number(n) => n,
                _ => {
                    return Err(JsonError::Invalid(format!(
                        "Key '{}' should be a number",
                        k
                    )))
                }
            };

            match address {
                1..=9999 | 10001..=19999 => {
                    let bit = number
                        .as_i64()
                        .filter(|&n| n == 0 || n == 1)
                        .ok_or_else(|| {
                            JsonError::Invalid(format!("Key '{}' should be 0 or 1", k))
                        })? as u16;

                    registers.insert(address, bit);
                }
                30001..=39999 | 40001..=49999 => {
                    let transformed = number.as_i64().ok_or(JsonError::Invalid(format!(
                        "Error parsing number at key '{}'",
                        address
                    )))?;

                    if let PackType::U64 | PackType::U32 | PackType::U16 = pack_type {
                        if transformed < 0 {
                            return Err(JsonError::Invalid(format!(
                                "Value at key '{}' should be positive",
                                address
                            )));
                        }
                    }

                    let bytes: Vec<u16> = Some(transformed)
                        .and_then(|n| match pack_type.len() {
                            1 => i16::try_from(n).map(|v| v.to_be_bytes().to_vec()).ok(),
                            2 => i32::try_from(n).map(|v| v.to_be_bytes().to_vec()).ok(),
                            4 => Some(n.to_be_bytes().to_vec()),
                            _ => unreachable!(),
                        })
                        .map(|vec| {
                            vec.chunks(2)
                                .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                                .collect()
                        })
                        .ok_or(JsonError::Invalid(format!(
                            "Error converting key {} to type {:?}",
                            address, pack_type
                        )))?;

                    for (idx, byte) in bytes.iter().enumerate() {
                        if registers.insert(address + idx as u16, *byte).is_some() {
                            return Err(JsonError::Invalid(format!(
                                "Overwrote register at key '{}'",
                                address
                            )));
                        }
                    }
                }
                other => {
                    return Err(JsonError::Invalid(format!(
                        "Key {} is outside modbus range",
                        other
                    )))
                }
            }
        }

        Ok((registers, keys))
    } else {
        return Err(JsonError::Invalid("data is not an object".into()));
    }
}

pub fn registers_to_object(
    registers: &HashMap<u16, u16>,
    keys: Vec<String>,
) -> Result<serde_json::Value, JsonError> {
    let mut json: Map<String, Value> = Map::new();

    for key in keys {
        let PackFormat { address, pack_type } = PackFormat::parse(key.as_str())
            .map_err(|_| JsonError::Other(format!("Failed to parse {}", key)))?;

        // Collect bytes based on specific addresses
        let bytes: Vec<&u16> = (address..address + pack_type.len() as u16)
            .filter_map(|addr| registers.get(&addr))
            .collect();

        if bytes.len() < pack_type.len() {
            return Err(JsonError::Other(format!(
                "Mismatching length at address {}",
                key
            )));
        }

        let number_str = match pack_type {
            PackType::U16 => (*bytes[0] as u16).to_string(),
            PackType::I16 => (*bytes[0] as i16).to_string(),

            // Concatenating two u16s to form a u32
            PackType::U32 => (((*bytes[0] as u32) << 16) | (*bytes[1] as u32)).to_string(),

            // Concatenating two u16s to form an i32
            PackType::I32 => ((((*bytes[0] as u32) << 16) | (*bytes[1] as u32)) as i32).to_string(),

            // Concatenating four u16s to form a u64
            PackType::U64 => (((*bytes[0] as u64) << 48)
                | ((*bytes[1] as u64) << 32)
                | ((*bytes[2] as u64) << 16)
                | (*bytes[3] as u64))
                .to_string(),

            // Concatenating four u16s to form an i64
            PackType::I64 => ((((*bytes[0] as u64) << 48)
                | ((*bytes[1] as u64) << 32)
                | ((*bytes[2] as u64) << 16)
                | (*bytes[3] as u64)) as i64)
                .to_string(),

            _ => return Err(JsonError::Other(format!("Error converting to type"))),
        };

        let value = serde_json::Number::from_str(number_str.as_str())
            .map_err(|_| JsonError::Other(format!("Error creating serde number value")))?;

        if json
            .insert(key.to_string(), serde_json::Value::Number(value))
            .is_some()
        {
            return Err(JsonError::Invalid(format!("Overwrote json map")));
        }
    }

    Ok(Value::Object(json))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use serde_json::json;
    type Error = Box<dyn std::error::Error>;

    #[test]
    pub fn test_parse() -> Result<(), Error> {
        let data = json!({
            "1": 1,
            "100": 1,
            "40001": 0,
            "40002": 0,
            "40003/h": 124,
            "40004": 124,
            "40005": 0,
            "40006": 0,
            "40007": 0,
            "40008": 32,
            "40100/h": -1,
            "40200/i": -10,
            "40300/q": -1,
        });

        let (registers, keys): (HashMap<u16, u16>, Vec<String>) =
            parse(data).map_err(|e| e.to_string())?;
        assert!(registers.get(&40003).unwrap() == &(124i16 as u16));
        assert!(registers.get(&40004).unwrap() == &(124i16 as u16));
        assert!(registers.get(&40100).unwrap() == &(-1i16 as u16));
        assert!(registers.get(&40008).unwrap() == &(32u16));
        assert_eq!(
            [
                *registers.get(&40200).unwrap(),
                *registers.get(&40201).unwrap(),
            ],
            [0xFFFF as u16, 0xFFF6 as u16]
        );
        assert_eq!(
            [
                *registers.get(&40300).unwrap(),
                *registers.get(&40301).unwrap(),
                *registers.get(&40302).unwrap(),
                *registers.get(&40303).unwrap(),
            ],
            [0xFFFF as u16, 0xFFFF as u16, 0xFFFF as u16, 0xFFFF as u16]
        );

        Ok(())
    }

    #[test]
    pub fn test_register_to_object() -> Result<(), Error> {
        let registers: HashMap<u16, u16> = HashMap::from([
            (1, -1i16 as u16),
            (2, 1),
            (3, 1),
            (9, 9),
            (40001, 0xFFFF),
            (40002, 0xFF9C),
            (40200, 0x0000),
            (40201, 0x0000),
            (40202, 0x0000),
            (40203, 0xFFFF),
        ]);

        let keys: Vec<String> = vec![
            "1/h".into(),
            "2".into(),
            "3".into(),
            "9".into(),
            "40001/i".into(),
            "40200/Q".into(),
        ];

        if let Value::Object(map) =
            registers_to_object(&registers, keys).map_err(|e| e.to_string())?
        {
            assert_eq!(map.get("1/h").unwrap().to_string(), "-1");
            assert_eq!(map.get("40001/i").unwrap().to_string(), "-100");
            assert_eq!(map.get("40200/Q").unwrap().to_string(), "65535")
        } else {
            assert!(false);
        };

        Ok(())
    }

    #[test]
    pub fn test_write() -> Result<(), Error> {
        let path = "test-output.json";
        let registers: HashMap<u16, u16> = HashMap::from([
            (1, -1i16 as u16),
            (2, 1),
            (3, 1),
            (9, 9),
            (40001, 0xFFFF),
            (40002, 0xFF9C),
            (40200, 0x0000),
            (40201, 0x0000),
            (40202, 0x0000),
            (40203, 0xFFFF),
        ]);
        let keys: Vec<String> = vec![
            "1/h".into(),
            "2".into(),
            "3".into(),
            "9".into(),
            "40001/i".into(),
            "40200/Q".into(),
        ];

        let value = registers_to_object(&registers, keys).unwrap();
        let value_str = serde_json::to_string_pretty(&value).unwrap();

        write(value, path).unwrap();

        let mut buf = String::new();
        File::open(path).unwrap().read_to_string(&mut buf).unwrap();

        assert_eq!(value_str, buf);

        fs::remove_file(path).unwrap();

        Ok(())
    }
}
