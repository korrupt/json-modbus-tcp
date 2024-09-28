#[derive(PartialEq, Debug)]
pub enum PackType {
    U16,
    I16,
    U32,
    I32,
    U64,
    I64
}

impl PackType {
    fn from_char(value: &u8) -> Option<Self> {
        match value {
            b'h' => Some(PackType::I16),
            b'H' => Some(PackType::U16),
            b'i' => Some(PackType::I32),
            b'I' => Some(PackType::U32),
            b'q' => Some(PackType::I64),
            b'Q' => Some(PackType::U64),
            _ => None
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
}


#[derive(PartialEq, Debug)]
pub struct PackFormat {
    pub address: u16,
    pub pack_type: PackType,
}


impl PackFormat {
    pub fn parse(addr: &str) -> Result<Self, PackError> {
        // Check if there's a '/' in the string
        if let Some(idx) = addr.find('/') {
            // Parse the address (before the '/')
            let address = addr[..idx].parse::<u16>().map_err(|_| PackError::Unsupported)?;

            // Get the part after the '/'
            addr.get(idx + 1..)
                .ok_or(PackError::Unsupported)  // Error if nothing after the '/'
                .and_then(|type_slice| match type_slice.as_bytes() {
                    // Check if it's a valid single character format
                    [format] => {
                        PackType::from_char(format)
                            .ok_or(PackError::Unsupported)  // Handle unsupported pack type
                    },
                    _ => Err(PackError::Unsupported),  // Error if invalid format
                })
                .map(|pack_type| PackFormat { address, pack_type })
        } else {
            // No '/', default to U16 and parse the address
            let address = addr.parse::<u16>().map_err(|_| PackError::Unsupported)?;
            Ok(PackFormat { address, pack_type: PackType::U16 })
        }
    }
}


#[derive(Debug, PartialEq)]
pub enum PackError {
    Unsupported
}



#[cfg(test)]
pub mod test {
    use crate::python_struct::*;

    #[test]
    pub fn test_packformat_parse() -> Result<(), anyhow::Error> {

        assert_eq!(PackFormat::parse("40001/h").unwrap(), PackFormat { address: 40001, pack_type: PackType::I16 });
        assert_eq!(PackFormat::parse("40311/H").unwrap(), PackFormat { address: 40311, pack_type: PackType::U16 });
        assert_eq!(PackFormat::parse("40311/i").unwrap(), PackFormat { address: 40311, pack_type: PackType::I32 });
        assert_eq!(PackFormat::parse("40311/I").unwrap(), PackFormat { address: 40311, pack_type: PackType::U32 });
        assert_eq!(PackFormat::parse("40311/<"), Err(PackError::Unsupported));
        assert_eq!(PackFormat::parse("40311").unwrap(), PackFormat { address: 40311, pack_type: PackType::U16});

        Ok(())
    }

}