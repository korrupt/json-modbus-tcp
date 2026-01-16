use std::{num::ParseIntError, str::FromStr};

use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, DeserializeFromStr, SerializeDisplay)]
pub enum PackType {
    U16,
    I16,
    U32,
    I32,
    U64,
    I64
}


impl PackType {
    pub fn payload_str(&self, words: &[u16]) -> String {
        match self {
            PackType::U16 => u16::from_be_words(words).to_string(),
            PackType::I16 => i16::from_be_words(words).to_string(),
            PackType::U32 => u32::from_be_words(words).to_string(),
            PackType::I32 => i32::from_be_words(words).to_string(),
            PackType::U64 => u64::from_be_words(words).to_string(),
            PackType::I64 => i64::from_be_words(words).to_string(),
        }
    }

    pub fn from_char(value: char) -> Option<Self> {
        match value {
            'h' => Some(PackType::I16),
            'H' => Some(PackType::U16),
            'i' => Some(PackType::I32),
            'I' => Some(PackType::U32),
            'q' => Some(PackType::I64),
            'Q' => Some(PackType::U64),
            _ => None
        }
    }

    pub fn to_char(&self) -> char {
        match &self {
            PackType::I16 => 'h',
            PackType::U16 => 'H',
            PackType::I32 => 'i',
            PackType::U32 => 'I',
            PackType::I64 => 'q',
            PackType::U64 => 'Q',
        }
    }

    pub fn len(&self) -> usize {
        match &self {
            PackType::U16 |
            PackType::I16 => 1,
            PackType::U32 |
            PackType::I32 => 2,
            PackType::U64 |
            PackType::I64 => 4,
        }
    }

    pub fn try_make_payload(&self, value: &str) -> Result<Vec<u16>, ParseIntError>{ 
        let bytes = match &self {
            PackType::U16 => {
                let val = value
                    .parse::<u16>()?;
                val.to_be_bytes().to_vec()
            }

            PackType::I16 => {
                let val = value
                    .parse::<i16>()?;
                val.to_be_bytes().to_vec()
            }

            PackType::U32 => {
                let val = value
                    .parse::<u32>()?;
                val.to_be_bytes().to_vec()
            }

            PackType::I32 => {
                let val = value
                    .parse::<i32>()?;
                val.to_be_bytes().to_vec()
            }

            PackType::U64 => {
                let val = value
                    .parse::<u64>()?;
                val.to_be_bytes().to_vec()
            }

            PackType::I64 => {
                let val = value
                    .parse::<i64>()?;
                val.to_be_bytes().to_vec()
            }
        };

        let mut words = Vec::with_capacity(self.len());
        for (_, chunk) in bytes.chunks_exact(2).enumerate() {
            let word = u16::from_be_bytes([chunk[0], chunk[1]]);
            words.push(word);
        }

        Ok(words)
    }
}

impl FromStr for PackType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "uint16" | "h" => Ok(PackType::U16),
            "int16" | "H" => Ok(PackType::I16),
            "uint32" | "i" => Ok(PackType::U32),
            "int32" | "I" => Ok(PackType::I32),
            "uint64" | "q" => Ok(PackType::U64),
            "int64" | "Q" => Ok(PackType::I64),
            _ => Err("Invalid format".into())
        }
    }
}

impl std::fmt::Display for PackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.to_char()))
    }
}

// Helper trait (you can reuse your previous one)
trait FromBeWords {
    fn from_be_words(words: &[u16]) -> Self;
}

macro_rules! impl_from_be_words {
    ($ty:ty, $words:literal) => {
        impl FromBeWords for $ty {
            fn from_be_words(words: &[u16]) -> Self {
                if words.len() != $words {
                    panic!("Expected {} words for {}, got {}", $words, stringify!($ty), words.len());
                }
                let mut val: Self = 0;
                for (i, &w) in words.iter().enumerate() {
                    val |= (w as Self) << (16 * ($words - 1 - i) as u32);
                }
                val
            }
        }
    };
}

impl_from_be_words!(u16, 1);
impl_from_be_words!(i16, 1);
impl_from_be_words!(u32, 2);
impl_from_be_words!(i32, 2);
impl_from_be_words!(u64, 4);
impl_from_be_words!(i64, 4);
