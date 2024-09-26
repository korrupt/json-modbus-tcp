#[derive(PartialEq, Debug)]

pub enum Endianness {
    BigEndian,
    LittleEndian
}

impl Endianness {
    pub fn from_char(value: &u8) -> Option<Self> {
        match value {
            b'<' => Some(Endianness::LittleEndian),
            b'>' => Some(Endianness::BigEndian),
            _ => None,
        }
    }
}

#[derive(PartialEq, Debug)]

enum PackType {
    U16,
    I16,
    U32,
    I32
}

impl PackType {
    fn from_char(value: &u8) -> Option<Self> {
        match value {
            b'h' => Some(PackType::I16),
            b'H' => Some(PackType::U16),
            b'i' => Some(PackType::I32),
            b'I' => Some(PackType::U32),
            _ => None
        }
    }
}


#[derive(PartialEq, Debug)]
pub struct PackFormat {
    endianness: Endianness,
    pack_type: PackType,
}

impl Default for PackFormat {
    fn default() -> Self {
        PackFormat {
            endianness: Endianness::BigEndian,
            pack_type: PackType::U16,
        }
    }
}

impl PackFormat {
    pub fn parse(addr: &str) -> Result<Self, PackError> {
        // check if there's any conf
        addr.find('/')
            .and_then(|idx| addr.get(idx + 1..))
            // error early if there's no characters after the '/'
            .ok_or(PackError::Unsupported)
            // parse whether theres 1 or 2 characters
            .and_then(|type_slice| match type_slice.as_bytes() {
                [endianness, format] => {
                    Endianness::from_char(endianness)
                        .and_then(|e| PackType::from_char(format).map(|p| (e, p)))
                        .ok_or(PackError::Unsupported)
                },
                [format] => {
                    PackType::from_char(format)
                        .map(|p| (Endianness::BigEndian, p))
                        .ok_or(PackError::Unsupported)
                },
                _ => Err(PackError::Unsupported),
            })
            .map(|(endianness, pack_type)| PackFormat { endianness, pack_type })
            .or_else(|_| Ok(Default::default()))
    }
}


#[derive(Debug)]
pub enum PackError {
    Unsupported
}



#[cfg(test)]
pub mod test {
    use crate::python_struct::*;

    #[test]
    pub fn test_32bit_int() -> Result<(), anyhow::Error> {

        assert_eq!(PackFormat::parse("40001/h").unwrap(), PackFormat { endianness: Endianness::BigEndian, pack_type: PackType::I16 });
        assert_eq!(PackFormat::parse("40311/H").unwrap(), PackFormat { endianness: Endianness::BigEndian, pack_type: PackType::U16 });
        assert_eq!(PackFormat::parse("40311/i").unwrap(), PackFormat { endianness: Endianness::BigEndian, pack_type: PackType::I32 });
        assert_eq!(PackFormat::parse("40311/>i").unwrap(), PackFormat { endianness: Endianness::BigEndian, pack_type: PackType::I32 });
        assert_eq!(PackFormat::parse("40311/<I").unwrap(), PackFormat { endianness: Endianness::LittleEndian, pack_type: PackType::U32 });

        Ok(())
    }

}