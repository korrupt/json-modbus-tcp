// ────────────────────────────────────────────────────────────────
// Trait definitions
#[allow(dead_code)]
pub trait AsWords<T> {
    fn as_words(&self) -> Vec<u16>;
}

#[allow(dead_code)]
pub trait FromVec<T> {
    fn from_vec(&self) -> T;
}

// ────────────────────────────────────────────────────────────────
// 16-bit versions (1 word)

impl AsWords<u16> for u16 {
    fn as_words(&self) -> Vec<u16> {
        vec![*self]
    }
}

impl AsWords<i16> for i16 {
    fn as_words(&self) -> Vec<u16> {
        vec![*self as u16]   // Bit cast - preserves bit pattern
    }
}

impl FromVec<u16> for Vec<u16> {
    fn from_vec(&self) -> u16 {
        if self.len() != 1 {
            panic!("Expected exactly 1 u16 word, got {}", self.len());
        }
        self[0]
    }
}

impl FromVec<i16> for Vec<u16> {
    fn from_vec(&self) -> i16 {
        if self.len() != 1 {
            panic!("Expected exactly 1 u16 word, got {}", self.len());
        }
        self[0] as i16   // Bit cast - preserves bit pattern
    }
}

// ────────────────────────────────────────────────────────────────
// 32-bit versions (2 words)

impl AsWords<u32> for u32 {
    fn as_words(&self) -> Vec<u16> {
        vec![
            ((self >> 16) & 0xFFFF) as u16,
            (self & 0xFFFF) as u16,
        ]
    }
}

impl AsWords<i32> for i32 {
    fn as_words(&self) -> Vec<u16> {
        vec![
            ((self >> 16) & 0xFFFF) as u16,
            (self & 0xFFFF) as u16,
        ]
    }
}

impl FromVec<u32> for Vec<u16> {
    fn from_vec(&self) -> u32 {
        if self.len() != 2 {
            panic!("Expected exactly 2 u16 words, got {}", self.len());
        }
        ((self[0] as u32) << 16) | (self[1] as u32)
    }
}

impl FromVec<i32> for Vec<u16> {
    fn from_vec(&self) -> i32 {
        if self.len() != 2 {
            panic!("Expected exactly 2 u16 words, got {}", self.len());
        }
        ((self[0] as i32) << 16) | (self[1] as i32)
    }
}

// ────────────────────────────────────────────────────────────────
// 64-bit versions (improved – more consistent style) – optional update

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

impl FromVec<u64> for Vec<u16> {
    fn from_vec(&self) -> u64 {
        if self.len() != 4 {
            panic!("Expected exactly 4 u16 words, got {}", self.len());
        }
        ((self[0] as u64) << 48)
            | ((self[1] as u64) << 32)
            | ((self[2] as u64) << 16)
            | (self[3] as u64)
    }
}

impl FromVec<i64> for Vec<u16> {
    fn from_vec(&self) -> i64 {
        if self.len() != 4 {
            panic!("Expected exactly 4 u16 words, got {}", self.len());
        }
        ((self[0] as i64) << 48)
            | ((self[1] as i64) << 32)
            | ((self[2] as i64) << 16)
            | (self[3] as i64)
    }
}