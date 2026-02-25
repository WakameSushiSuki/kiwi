use lexeme::*;

#[derive(Clone, Copy)]
pub struct Span {
    pub line: u32,
    pub col: u16
}

impl Span {
    pub fn new(line: u32, col: u16) -> Self {
        Self { line: line, col: col }
    }

    pub fn empty() -> Self {
        Self::new(0, 0)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum TokenType {
    Empty, Ident, Int, Float,
    Char, Str,
}

#[derive(Clone, Copy)]
pub struct Token<'a> {
    pub lexeme: Lexeme<'a>,
    pub tag: TokenType,
    pub span: Span,
}

impl<'a> Token<'a> {
    pub fn new(
        lexeme: Lexeme<'a>,
        tag: TokenType,
        span: Span
    ) -> Self {
        Self {
            lexeme: lexeme,
            tag: tag,
            span: span,
        }
    }

    pub fn empty() -> Self {
        Self::new(
            Lexeme::empty(),
            TokenType::Empty,
            Span::empty()
        )
    }
}

