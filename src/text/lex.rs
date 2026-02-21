use std::slice;
use std::collections::HashSet;

#[derive(Clone, Copy)]
union LexemeData {
    raw: u64,
    ptr: *const u8,
}

pub enum LexemeValue<'a> {
    Inlined([u8; 8], usize),
    Slice(&'a [u8]),
}

#[derive(Clone, Copy)]
pub struct Lexeme {
    data: LexemeData,
    len: usize,
}

impl Lexeme {
    pub fn as_ptr(&self) -> *const u8 {
        unsafe { self.data.ptr }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_inlined(&self) -> bool {
        (unsafe { self.data.raw } & 1) != 0
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data.ptr, self.len) }
    }

    pub fn as_inline(&self) -> [u8; 8] {
        (unsafe { self.data.raw } >> 1).to_be_bytes()
    }

    pub fn as_value(&self) -> LexemeValue<'_> {
        if self.is_inlined() {
            LexemeValue::Inlined(self.as_inline(), self.len)
        } else {
            LexemeValue::Slice(self.as_slice())
        }
    }
}

impl From<&str> for Lexeme {
    fn from(item: &str) -> Self {
        let len = item.len();

        if item.is_ascii() && len <= 8 {
            let mut bytes = [0u8; 8];
            bytes[..len].copy_from_slice(item.as_bytes());

            Lexeme {
                data: LexemeData { raw: u64::from_be_bytes(bytes) },
                len,
            }
        } else {
            Lexeme {
                data: LexemeData { ptr: item.as_ptr() },
                len,
            }
        }
    }
}
#[repr(u8)]
pub enum TokenType {
    Empty, Ident, Int, Float,
    Char, Str,
}

#[derive(Clone, Copy)]
pub struct Token {
    pub lexeme: Lexeme,
    pub tag: TokenType,
}
