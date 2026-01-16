use std::{collections::{HashMap, HashSet}, ops::{Range, RangeInclusive}, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize};
use serde_with::serde_as;
use thiserror::Error;

use crate::pack::PackType;


pub type Register = HashMap<u16, u16>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde_as]
pub struct RegisterDefinition {
    #[serde(default)]
    pub offset: u16,
    #[serde(rename = "type", default = "default_data_type")]
    pub data_type: PackType,
    #[serde(default = "default_default_value")]
    pub value: String,
}

impl RegisterDefinition {

    pub fn to_range(&self) -> Range<u16> {
        self.offset..self.offset+self.data_type.len() as u16
    }

    pub fn to_payload(&self) -> Result<Vec<u16>, RegisterError> {
        self.data_type
            .try_make_payload(self.value.as_str())
            .map_err(|e| RegisterError::Config(format!("Register {}: {}", self.offset, e)))
    }
    
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum RegisterType {
    Inputs,
    Coils,
    HoldingRegisters,
    InputRegisters,
}

#[allow(dead_code)]
impl RegisterType {
    pub fn range(&self) -> RangeInclusive<u16> {
        match self {
            RegisterType::Coils => 1..=9999,
            RegisterType::Inputs => 10000..=19999,
            RegisterType::InputRegisters => 30000..=39999,
            RegisterType::HoldingRegisters => 40000..=49999
        }
    }

    pub fn from_key(key: &u16) -> Option<Self> {
        match key {
            1..=9999 => Some(RegisterType::Coils),
            10001..=19999 => Some(RegisterType::Inputs),
            30001..=39999 => Some(RegisterType::InputRegisters),
            40001..=49999 => Some(RegisterType::HoldingRegisters),
            _ => None
        }
    }

    pub fn try_from_range(range: &Range<u16>) -> Option<RegisterType>
    {
        let r = range;
        let t = Self::from_key(&r.start);
        
        if r.clone().skip(1).all(|x| Self::from_key(&x) == t) {
            Some(t?)
        } else {
            None
        }
    }
}

pub struct Canonical<'a> {
    pub definitions: &'a [RegisterDefinition]
}

impl Canonical<'_> {
    pub fn try_from_definition<'a>(definitions: &'a [RegisterDefinition]) -> Result<Canonical<'a>, RegisterError>
    {
        let mut set = HashSet::new();

        for def in definitions {
            let range = def.to_range();
            let reg_type = RegisterType::try_from_range(&range).ok_or(RegisterError::Config(format!("Register at {} out of bounds", range.start)))?;

            if matches!(reg_type, RegisterType::Coils)
                && !matches!(def.value.as_str(), "0" | "1") {
                    return Err(RegisterError::Config(format!("Coil at {} can only be 1 or 0", range.start)))
                }

            if matches!(reg_type, RegisterType::Coils | RegisterType::Inputs)
                && def.data_type.len() > 1 {
                    return Err(RegisterError::Config(format!("Register at {} has to be int16 or u16", range.start)));
                }

            if matches!(def.data_type, PackType::U16 | PackType::U32 | PackType::U64) {
                let int = def.value.parse::<i64>().map_err(|_| RegisterError::Config(format!("Could not parse default value at for register {}", range.start)))?;

                if int < 0 {
                    return Err(RegisterError::Config(format!("Default value for register {} has to be > 0", range.start)));
                }
            }

            for k in range {
                if !set.insert(k) { return Err(RegisterError::Config(format!("Overwrote register at {}", k))) }                
            }
        }

        Ok(Canonical { definitions })
    }
}

impl<'a> Canonical<'a> {

    pub fn to_register(&self) -> Register {
        let size: usize = self.definitions.iter().map(|d| d.data_type.len()).sum();
        let mut register: Register = HashMap::with_capacity(size);

        for def in self.definitions {
            let words = def.to_payload().expect("This should never fail");
            for (i, word) in words.into_iter().enumerate() {
                register.insert(def.offset + i as u16, word);
            }
        }

        register
    }

}

#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    ParseError(String)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum RegisterEntry {
    Structured(RegisterDefinition),
    Compact(String),
}

impl RegisterEntry {
    fn into_definition(self) -> Result<RegisterDefinition, String> {
        match self {
            Self::Compact(s) => s.parse(),
            Self::Structured(def) => Ok(def)
        }
    }
}

fn default_data_type() -> PackType {
    PackType::U16
}

fn default_default_value() -> String {
    "0".into()
}


impl ToString for RegisterDefinition {
    fn to_string(&self) -> String {
        format!("{}/{}: {}", self.offset, self.data_type.to_char(), self.value.clone())
    }
}



impl FromStr for RegisterDefinition {
    type Err = String; 
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        let split_pos = s.rfind(':').zip(s.rfind('='))
            .map(|(colon, equal)| colon.max(equal))
            .or_else(|| s.rfind(':'))
            .or_else(|| s.rfind('='));

        let (before_default, default_str) = match split_pos {
            Some(pos) => {
                let (left, _) = s.split_at(pos);
                let sep_len = if s.as_bytes().get(pos) == Some(&b'=') { 1 } else { 1 };
                (left.trim(), s[pos + sep_len..].trim())
            }
            None => (s.trim(), "0"),
        };

        let value = default_str.to_string();

        let (offset_str, type_str) = match before_default.rsplit_once('/') {
            Some((off, typ)) => (off.trim(), typ.trim()),
            None => (before_default.trim(), ""),
        };

        let offset: u16 = offset_str
            .parse()
            .map_err(|_| format!("Invalid offset: '{offset_str}'"))?;

        let data_type = if type_str.is_empty() {
            PackType::U16
        } else if type_str.len() == 1 {
            PackType::from_char(type_str.chars().next().unwrap())
                .ok_or_else(|| format!("Unknown type specifier: '{type_str}'"))?
        } else {
            return Err(format!("Invalid type format: '{type_str}' (expected single char or empty)"));
        };

        Ok(RegisterDefinition {
            offset,
            data_type,
            value,
        })
    }
}

pub fn deserialize_register_definintions<'de, D>(deserializer: D) -> Result<Vec<RegisterDefinition>, D::Error>
    where D: Deserializer<'de>,
{
    let raw: Vec<RegisterEntry> = Vec::deserialize(deserializer)?;

    raw.into_iter()
        .enumerate()
        .map(|(idx, entry)| {{
            entry
                .into_definition()
                .map_err(|e| serde::de::Error::custom(format!("Invalid register at index {}: {}", idx, e)))
        }})
        .collect()
}

#[cfg(test)]
pub mod test {
    use std::str::FromStr;

    use crate::register::RegisterDefinition;
    use crate::pack::PackType;


    type Error = Box<dyn std::error::Error>;
    #[test]
    pub fn test_register_definition_from_str() -> Result<(), Error> {

        let coil_def = RegisterDefinition {
            data_type: PackType::U16,
            value: "0".into(),
            offset: 1
        };

        assert_eq!(RegisterDefinition::from_str("1").unwrap(), coil_def);
        assert_eq!(RegisterDefinition::from_str("1/H").unwrap(), coil_def);
        assert_eq!(RegisterDefinition::from_str("1:0").unwrap(), coil_def);
        assert_eq!(RegisterDefinition::from_str("1/H:0").unwrap(), coil_def);

        let reg_def = RegisterDefinition {
            data_type: PackType::U16,
            value: "0".into(),
            offset: 40007
        };

        assert_eq!(RegisterDefinition::from_str("40007").unwrap(), reg_def);
        assert_eq!(RegisterDefinition::from_str("40007:0").unwrap(), reg_def);
        assert_eq!(RegisterDefinition::from_str("40007/H").unwrap(), reg_def);
        assert_eq!(RegisterDefinition::from_str("40007/H=0").unwrap(), reg_def);

        let reg_int = RegisterDefinition {
            data_type: PackType::I64,
            value: "-64".into(),
            offset: 40016
        };

        assert_eq!(RegisterDefinition::from_str("40016/q=-64").unwrap(), reg_int);
    
        Ok(())
    }

}