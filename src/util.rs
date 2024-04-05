pub trait AsWords<T> {
    fn as_words(&self) -> Vec<u16>;
}

pub trait FromVec<T> {
    fn from_vec(&self) -> T;
}

impl FromVec<u64> for Vec<u16> {
    fn from_vec(&self) -> u64 {
        if self.len() != 4 {
            panic!("Invalid vec");
        }

        let msb = (self[0] as u64) << 48;
        let second = (self[1] as u64) << 32;
        let third = (self[2] as u64) << 16;
        let lsb = self[3] as u64;
        

        msb | second | third | lsb
    }
}

impl FromVec<i64> for Vec<u16> {
    fn from_vec(&self) -> i64 {
        if self.len() != 4 {
            panic!("Invalid vec");
        }

        let msb = (self[0] as i64) << 48;
        let second = (self[1] as i64) << 32;
        let third = (self[2] as i64) << 16;
        let lsb = self[3] as i64;
        

        msb | second | third | lsb
    }
}

impl AsWords<u64> for u64 {
    fn as_words(&self) -> Vec<u16> {
        vec![
            ((self >> 48) & 0xFFFF) as u16,
            ((self >> 32) & 0xFFFF) as u16,
            ((self >> 16) & 0xFFFF) as u16,
            (self & 0xFFFF) as u16,
        ]
    }
}

impl AsWords<i64> for i64 {
    fn as_words(&self) -> Vec<u16> {
        vec![
            ((self >> 48) & 0xFFFF) as u16,
            ((self >> 32) & 0xFFFF) as u16,
            ((self >> 16) & 0xFFFF) as u16,
            (self & 0xFFFF) as u16,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type Error = Box<dyn std::error::Error>;

    #[test]
    pub fn test_u64() -> Result<(), Error> {

        let num = 12u64;

        let as_words = num.as_words();

        assert_eq!(as_words, vec![0x0000, 0x0000, 0x0000, 0x000C]);

        let from_vec: u64 = as_words.from_vec();

        assert_eq!(from_vec, 12u64);

        Ok(())
    }

    #[test]
    pub fn test_i64() -> Result<(), Error> {

        let num = -14i64;

        let as_words = num.as_words();

        assert_eq!(as_words, vec![0xFFFF, 0xFFFF, 0xFFFF, 0xFFF2]);

        let from_vec: i64 = as_words.from_vec();

        assert_eq!(from_vec, -14i64);

        Ok(())
    }
}