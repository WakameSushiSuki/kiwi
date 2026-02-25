pub struct Cursor<'a> {
    contents: &'a str,
    ptr: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(contents: &'a str) -> Cursor<'a> {
        Cursor { contents, ptr: 0 }
    }

    pub fn is_eof(&self) -> bool {
        self.ptr >= self.contents.len()
    }

    pub fn remaining_len(&self) -> usize{
        self.contents.len() - self.ptr
    }

    pub fn remaining_data(&self) -> &'a str {
        &self.contents[self.ptr..]
    }

    pub fn position(&self) -> usize {
        self.ptr
    }

    pub fn set_position(&mut self, pos: usize) {
        assert!(pos <= self.contents.len());
        assert!(self.contents.is_char_boundary(pos));

        self.ptr = pos;
    }

    pub fn move_by(&mut self, n_bytes: usize) -> bool {
        let new = self.ptr + n_bytes;
        if new <= self.contents.len() && self.contents.is_char_boundary(new) {
            self.ptr = new;
            true
        } else {
            false
        }
    }

    pub fn peek(&self, n_bytes: usize) -> Option<&'a str> {
        let end = self.ptr.checked_add(n_bytes)?;
        if end <= self.contents.len() && self.contents.is_char_boundary(end) {
            Some(&self.contents[self.ptr..end])
        } else {
            None
        }
    }

    pub fn peek_char(&self) -> Option<char> {
        self.remaining_data().chars().next()
    }

    pub fn take(&mut self, n_bytes: usize) -> Option<&'a str> {
        let result = self.peek(n_bytes)?;
        self.ptr += n_bytes;
        Some(result)
    }

    pub fn take_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.ptr += ch.len_utf8();
        Some(ch)
    }

    pub fn take_while<F>(&mut self, mut f: F) -> &'a str
    where
        F: FnMut(char) -> bool,
    {
        let start = self.ptr;

        while let Some(ch) = self.peek_char() {
            if f(ch) {
                self.ptr += ch.len_utf8();
            } else {
                break;
            }
        }

        &self.contents[start..self.ptr]
    }

    //TODO: Error handling
    pub fn expect<'b: 'a>(&mut self, expected: &'b str) -> Result<&'a str, ()> {
        if self.remaining_data().starts_with(expected) {
            self.ptr += expected.len();
            Ok(expected)
        } else {
            Err(()) // ERROR: unexpected token
        }
    }

    //TODO: Error handling
    pub fn expect_char(&mut self, expected: char) -> Result<char, ()> {
        match self.take_char() {
            Some(c) if c == expected => Ok(c),
            _ => Err(()) // ERROR: unexpected char
        }
    }

    pub fn read_token<T: ToToken<'a>>(&mut self) -> Result<T, ()>{
        let old_pos = self.position();
        match T::to_token(self) {
            Ok(v) => Ok(v),
            Err(r) => {self.set_position(old_pos); Err(r)}
        }
    }
}

pub trait ToToken<'a> where Self: Sized{
    //TODO: Error handling
    fn to_token(c: &mut Cursor<'a>) -> Result<Self, ()>;
}
//TODO: Make a proc macro?
macro_rules! Token {
    ($val:literal => $name:ident) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        struct $name{
            //TODO: Spans
        } impl<'a> ToToken<'a> for $name{
            fn to_token(c: &mut Cursor<'a>) -> Result<$name, ()> {
                c.expect($val)?;
                Ok($name{})
            }
        }
    };

    ([$v:vis] $val:literal => $name:ident) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        $v struct $name{
            //TODO: Spans
        } impl<'a> ToToken<'a> for $name{
            fn to_token(c: &mut Cursor<'a>) -> Result<$name, ()> {
                c.expect($val)?;
                Ok($name{})
            }
        }
    };

    ($($vals:literal => $names:ident),+) => {
        $(Token!{$vals => $names})*
    };

    ($([$v:vis] $vals:literal => $names:ident),+) => {
        $(Token!{$v @ $vals => $names})*
    };

    (enum $ename:ident{$($names:ident = $vals:literal),+}) => {
        $(Token!{$vals => $names})*

        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        enum $ename{
            $($names($names)),+
        } impl<'a> ToToken<'a>  for $ename{
            fn to_token(c: &mut Cursor<'a>) -> Result<$ename, ()> {
                $(
                    if let Ok(tk) = c.read_token::<$names>(){
                        return Ok($ename::$names(tk))
                    }
                )+
                Err(()) // ERROR: Expected one of [variants...]
            }
        }
    };

    ($v:vis enum $ename:ident{$($names:ident = $vals:literal),+}) => {
        $(Token!{[$v] $vals => $names})*

        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        $v enum $ename{
            $($names($names)),+
        } impl<'a> ToToken<'a> for $ename{
            fn to_token(c: &mut Cursor<'a>) -> Result<$ename, ()> {
                $(
                    if let Ok(tk) = c.read_token::<$names>(){
                        return Ok($ename::$names(tk))
                    }
                )+
                Err(()) // ERROR: Expected one of [variants...]
            }
        }
    };
}

Token!{
    pub enum Keyword{
        If = "if",
        While = "while",
        Else = "else",

        Import = "import",

        Static = "static",
        Constant = "const"
    }
}

Token!{
    pub enum Symbol{
        Inc = "++",
        Dec = "--",
        
        ShiftLeft = "<<",
        ShiftRight = ">>",

        Equals = "==",
        NotEquals = "!=",
        GreaterThanOrEq = ">=",
        GreaterThan = ">",
        LessThanOrEq = "<=",
        LessThan = "<",

        DoubleColon = "::",
        SemiColon = ";",

        LeftParen = "(",
        RightParen = ")",
        LeftSquare = "[",
        RightSquare = "]",
        LeftCurly = "{",
        RightCurly = "}",

        Add = "+",
        Subtract = "-",
        Multiply = "*",
        Divide = "/",
        Modulus = "%",

        Assign = "=",

        Comma = ",",

        Period = "."

    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident{
    data: String
    //TODO: Spans
} impl<'a> ToToken<'a> for Ident {
    fn to_token(c: &mut Cursor<'a>) -> Result<Self, ()> {
        let f = c.take_while(|x| x.is_alphabetic() || x == '_');
        if f.len() == 0{
            Err(()) // Error no data
        } else {
            Ok(Ident{data: f.to_string() + c.take_while(|x| x.is_alphanumeric() || x == '_')})
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    Char(char),
    String(String)
} impl Literal {
    fn read_integer<'a>(c: &mut Cursor<'a>) -> Result<Self, ()> {
        let mut s = String::new();

        if let Some('-') = c.peek_char() {
            s.push(c.take_char().unwrap());
        }

        let digits = c.take_while(|ch| ch.is_ascii_digit());

        if digits.is_empty() {
            return Err(());
        }

        s.push_str(digits);

        match s.parse::<i64>() {
            Ok(n) => Ok(Literal::Integer(n)),
            Err(_) => {
                Err(())
            }
        }
    }

    fn read_float<'a>(c: &mut Cursor<'a>) -> Result<Self, ()> {
        let mut s = String::new();

        if let Some('-') = c.peek_char() {
            s.push(c.take_char().unwrap());
        }

        let int_part = c.take_while(|ch| ch.is_ascii_digit());
        s.push_str(int_part);

        if c.peek_char() != Some('.') {
            return Err(());
        }

        s.push(c.take_char().unwrap());

        let frac_part = c.take_while(|ch| ch.is_ascii_digit());
        if frac_part.is_empty() {
            return Err(());
        }

        s.push_str(frac_part);

        if matches!(c.peek_char(), Some('e') | Some('E')) {
            s.push(c.take_char().unwrap());

            if matches!(c.peek_char(), Some('+') | Some('-')) {
                s.push(c.take_char().unwrap());
            }

            let exp_digits = c.take_while(|ch| ch.is_ascii_digit());
            if exp_digits.is_empty() {
                return Err(());
            }

            s.push_str(exp_digits);
        }

        match s.parse::<f64>() {
            Ok(f) => Ok(Literal::Float(f)),
            Err(_) => {
                Err(())
            }
        }
    }

    fn read_char<'a>(c: &mut Cursor<'a>) -> Result<Self, ()> {
        if c.take_char() != Some('\'') {
            return Err(());
        }

        let ch = match c.take_char() {
            Some('\\') => match c.take_char() {
                Some('n') => '\n',
                Some('t') => '\t',
                Some('r') => '\r',
                Some('0') => '\0',
                Some('\\') => '\\',
                Some('\'') => '\'',
                Some('"') => '"',
                _ => {
                    return Err(());
                }
            },
            Some(c) => c,
            None => {
                return Err(());
            }
        };

        if c.take_char() != Some('\'') {
            return Err(());
        }

        Ok(Literal::Char(ch))
    }

    fn read_string<'a>(c: &mut Cursor<'a>) -> Result<Self, ()> {
        if c.take_char() != Some('"') {
            return Err(());
        }

        let mut result = String::new();

        while let Some(ch) = c.take_char() {
            match ch {
                '"' => return Ok(Literal::String(result)),
                '\\' => {
                    let escaped = match c.take_char() {
                        Some('n') => '\n',
                        Some('t') => '\t',
                        Some('r') => '\r',
                        Some('0') => '\0',
                        Some('\\') => '\\',
                        Some('\'') => '\'',
                        Some('"') => '"',
                        _ => {
                            return Err(());
                        }
                    };
                    result.push(escaped);
                }
                _ => result.push(ch),
            }
        }
        Err(())
    }
} impl<'a> ToToken<'a> for Literal  {
    fn to_token(c: &mut Cursor<'a>) -> Result<Self, ()> {
        match c.peek_char() {
            Some('-') | Some('0'..='9') => {
                let p = c.position();
                if let Ok(r) = Literal::read_float(c){
                    Ok(r)
                } else {
                    c.set_position(p);
                    Literal::read_integer(c)
                }
            }
            Some('\'') => Literal::read_char(c),
            Some('"') => Literal::read_string(c),
            Some(_) => Err(()), // Invalid
            None => Err(()) // No data
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Symbol(Symbol),
    Ident(Ident),
    Literal(Literal)
} impl <'a> ToToken<'a> for Token{
    fn to_token(c: &mut Cursor<'a>) -> Result<Self, ()> {
        //TODO: fix this
        if let Ok(s) = c.read_token(){
            return Ok(Token::Symbol(s))
        }
        match c.peek_char() {
            Some('-') | Some('0'..='9') | Some('\'') | Some('"') => Ok(Token::Literal(c.read_token()?)),
            Some(ch) if ch.is_alphabetic() => {
                if let Ok(kw) = c.read_token(){
                    Ok(Token::Keyword(kw))
                } else if let Ok(id) = c.read_token(){
                    Ok(Token::Ident(id))
                } else {
                    unreachable!()
                }
            }
            Some(_) => Err(()), // Invalid
            None => Err(()) // No data
        }
    }
}