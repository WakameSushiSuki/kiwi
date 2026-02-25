use std::{marker::PhantomData, slice};

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
pub struct Lexeme<'a> {
    data: LexemeData,
    len: usize,
    _marker: PhantomData<&'a u8>,
}

impl<'a> Lexeme<'a> {
    pub fn empty() -> Self {
        Self { data: LexemeData { raw: 1 }, len: 0, _marker: PhantomData}
    }

    pub unsafe fn as_ptr(&self) -> *const u8 {
        unsafe { self.data.ptr }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_inlined(&self) -> bool {
        (unsafe { self.data.raw } & 1) != 0
    }

    pub unsafe fn as_raw_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data.ptr, self.len) }
    }

    pub unsafe fn as_raw_inline(&self) -> [u8; 8] {
        (unsafe{ self.data.raw } >> 1).to_be_bytes()
    }

    pub fn as_value(&self) -> LexemeValue<'_> {
        if self.is_inlined() {
            LexemeValue::Inlined(unsafe{ self.as_raw_inline() }, self.len)
        } else {
            LexemeValue::Slice(unsafe{ self.as_raw_slice() })
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            if self.is_inlined() {
                // Data is truncated to len
                let ptr = &self.data.raw as *const u64 as *const u8;
                std::slice::from_raw_parts(ptr, self.len)
            } else {
                self.as_raw_slice()
            }
        }
    }

    pub fn as_str(&self) -> &str {
        // UTF-8 validated before or during lexing, and inlined strings
        // are always ASCII
        unsafe { str::from_utf8_unchecked(self.as_slice()) }
    }
}

impl<'a> From<&str> for Lexeme<'a> {
    fn from(item: &str) -> Self {
        let len = item.len();

        if item.is_ascii() && len <= 8 {
            let mut bytes = [0u8; 8];
            bytes[..len].copy_from_slice(item.as_bytes());

            Lexeme {
                data: LexemeData { raw: (u64::from_be_bytes(bytes) << 1) | 1 },
                len,
                _marker: PhantomData
            }
        } else {
            Lexeme {
                data: LexemeData { ptr: item.as_ptr() },
                len,
                _marker: PhantomData
            }
        }
    }
}

impl<'a> AsRef<[u8]> for Lexeme<'a> {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<'a> AsRef<str> for Lexeme<'a> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

