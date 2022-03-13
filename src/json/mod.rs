mod lexer;
pub use lexer::Lexer;

mod parser;
pub use parser::Parser;

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum KeyPart {
    Identifier(String),
    Index(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Key {
    depth: usize,
    parts: Vec<KeyPart>,
}

impl Default for Key {
    fn default() -> Self {
        Self {
            depth: 0,
            parts: Vec::new(),
        }
    }
}

impl Key {
    pub fn new(parts: Vec<KeyPart>) -> Self {
        Self { depth: 0, parts }
    }

    pub fn is_empty(&self) -> bool {
        let len = self.parts.len();
        if len == 0 {
            false
        } else {
            len - 1 == self.depth
        }
    }

    pub fn first_part(&self) -> &KeyPart {
        self.parts.iter().skip(self.depth).next().unwrap()
    }

    pub fn descend(&mut self) -> bool {
        if !self.is_empty() {
            self.depth += 1;
            true
        } else {
            false
        }
    }

    pub fn ascend(&mut self) -> bool {
        if self.is_empty() {
            self.depth -= 1;
            true
        } else {
            false
        }
    }

    #[must_use]
    pub fn copy_of(&self) -> Self {
        let Key { parts, .. } = self;
        let mut cloned = Key {
            depth: 0,
            parts: parts.clone(),
        };
        cloned.depth = 0;
        cloned
    }

    #[must_use]
    pub fn push_str(mut self, string: &str) -> Self {
        self.parts.push(KeyPart::Identifier(string.to_string()));
        self
    }

    #[must_use]
    pub fn push_idx(mut self, index: usize) -> Self {
        self.parts.push(KeyPart::Index(index));
        self
    }

    pub fn pop(&mut self) -> Option<KeyPart> {
        if self.is_empty() && self.depth != 0 {
            self.depth -= 1;
        }
        self.parts.pop()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Object(HashMap<String, Json>),
    Array(Vec<Json>),
    Number {
        integer: i64,
        fraction: (u32, u64),
        exponent: i64,
    },
    String(String),
    Boolean(bool),
    Null,
}

impl From<&str> for Json {
    fn from(input: &str) -> Self {
        Self::String(input.to_string())
    }
}

impl Json {
    pub fn get(&self, key: &mut Key) -> Option<&Self> {
        let first_part = key.first_part();

        let object = match (self, first_part) {
            (Json::Object(obj), KeyPart::Identifier(key)) => obj.get(key),
            (Json::Array(arr), KeyPart::Index(idx)) => arr.get(*idx),
            _ => None,
        };

        if key.descend() {
            if let Some(object) = object {
                object.get(key)
            } else {
                None
            }
        } else {
            object
        }
    }

    pub fn to_string(&self) -> String {
        let mut string = String::new();

        self.to_string_rec(&mut string);

        string
    }

    pub fn from_string<'input>(input: &'input str) -> Option<Json> {
        let mut tokens = Vec::new();

        if let Err(_) = Lexer::new(Some(input)).lex_into(input.chars(), &mut tokens) {
            return None;
        }

        if let Ok(value) = Parser::parse_tokens(&tokens) {
            value
        } else {
            None
        }
    }

    fn to_string_rec(&self, buffer: &mut String) {
        match self {
            Json::Object(map) => {
                buffer.push('{');
                let mut peekable = map.iter().peekable();
                while let Some((key, value)) = peekable.next() {
                    buffer.push_str(format!("\"{}\":", key).as_str());
                    value.to_string_rec(buffer);
                    if peekable.peek().is_some() {
                        buffer.push(',');
                    }
                }
                buffer.push('}');
            }
            Json::Array(array) => {
                buffer.push('[');
                let mut peekable = array.iter().peekable();
                while let Some(next) = peekable.next() {
                    next.to_string_rec(buffer);
                    if peekable.peek().is_some() {
                        buffer.push(',');
                    }
                }
                buffer.push(']');
            }
            Json::Number {
                integer,
                fraction: (leading_zeroes, frac_number),
                exponent,
            } => {
                let value = match (frac_number, exponent) {
                    (0, 0) => format!("{}", integer),
                    (frac, 0) => format!(
                        "{}.{}{}",
                        integer,
                        (0..*leading_zeroes).map(|_| '0').collect::<String>(),
                        frac
                    ),
                    (0, exp) => format!("{}e{}", integer, exp),
                    (frac, exp) => {
                        format!(
                            "{}.{}{}e{}",
                            integer,
                            (0..*leading_zeroes).map(|_| '0').collect::<String>(),
                            frac,
                            exp
                        )
                    }
                };
                buffer.push_str(value.as_str())
            }
            Json::String(string) => buffer.push_str(format!("\"{}\"", string).as_str()),
            Json::Boolean(bool) => {
                if *bool {
                    buffer.push_str("true")
                } else {
                    buffer.push_str("false")
                }
            }
            Json::Null => buffer.push_str("null"),
        }
    }
}
