use std::{fmt::Display, iter::Peekable, str::Chars};

use TokenizeError::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenizeError<'src> {
    EOF(Span<'src>),
    InvalidLiteral(Span<'src>),
    NewlineInString(Span<'src>),
    InvalidEscape(Span<'src>),
    UnterminatedString(Span<'src>),
    IllegalWhitespace(Span<'src>),
}

impl<'src> TokenizeError<'src> {
    pub fn span(&self) -> &Span<'src> {
        match self {
            EOF(span)
            | InvalidLiteral(span)
            | NewlineInString(span)
            | InvalidEscape(span)
            | UnterminatedString(span)
            | IllegalWhitespace(span) => span,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Span<'src> {
    source: Option<&'src str>,
    source_offset: usize,
    line: usize,
    line_offset: usize,
    len: usize,
}

impl<'src> std::fmt::Debug for Span<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Span")
            .field("source_offset", &self.source_offset)
            .field("line", &self.line)
            .field("line_offset", &self.line_offset)
            .field("len", &self.len)
            .finish()
    }
}

impl<'src> Span<'src> {
    pub fn new(
        source: Option<&'src str>,
        source_offset: usize,
        line: usize,
        line_offset: usize,
        len: usize,
    ) -> Self {
        Self {
            source,
            source_offset,
            line,
            line_offset,
            len,
        }
    }

    pub fn lexeme(&self) -> Option<String> {
        self.source.map(|val| {
            val.chars()
                .skip(self.source_offset)
                .take(self.len)
                .collect()
        })
    }

    pub fn line_offset(&self) -> usize {
        self.line_offset
    }

    pub fn source(&self) -> Option<&str> {
        self.source
    }

    pub fn len(&self) -> usize {
        self.len
    }

    fn inc_ptr(&mut self, newline: bool) {
        if !newline {
            self.line_offset += 1;
        } else {
            self.line_offset = 0;
            self.line += 1;
        }
        self.source_offset += 1;
    }
}

impl<'src> Display for Span<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(lexeme) = self.lexeme() {
            write!(f, "{}", lexeme)
        } else {
            write!(f, "Character {} of input", self.source_offset)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal<'src> {
    Number(Span<'src>, String),
    String(Span<'src>, String),
    True(Span<'src>),
    False(Span<'src>),
    Null(Span<'src>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'src> {
    Whitespace(Span<'src>),
    Literal(Literal<'src>),
    ObjectStart(Span<'src>),
    ObjectEnd(Span<'src>),
    ArrayStart(Span<'src>),
    ArrayEnd(Span<'src>),
    Comma(Span<'src>),
    Colon(Span<'src>),
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.span())
    }
}

impl<'src> Token<'src> {
    pub fn span(&self) -> Span<'src> {
        match self {
            Token::Whitespace(span)
            | Token::ObjectStart(span)
            | Token::ObjectEnd(span)
            | Token::ArrayStart(span)
            | Token::ArrayEnd(span)
            | Token::Comma(span)
            | Token::Colon(span) => span.clone(),
            Token::Literal(id) => match id {
                Literal::String(span, _)
                | Literal::Null(span)
                | Literal::False(span)
                | Literal::True(span)
                | Literal::Number(span, _) => span.clone(),
            },
        }
    }
}

pub struct Lexer<'src> {
    current_loc: Span<'src>,
}

type LiteralResult<'src> = Result<Literal<'src>, TokenizeError<'src>>;
type SpanResult<'src> = Result<Span<'src>, TokenizeError<'src>>;
type TokenizeResult<'src> = Result<Vec<Token<'src>>, TokenizeError<'src>>;

impl<'src> Lexer<'src> {
    pub fn new(source: Option<&'src str>) -> Self {
        Self {
            current_loc: Span::new(source, 0, 0, 0, 0),
        }
    }

    pub fn lex_str(input: &'src str) -> TokenizeResult<'src> {
        let me = Self::new(Some(input));
        let mut tokens = Vec::new();
        me.lex_into(input.chars(), &mut tokens)?;
        Ok(tokens)
    }

    pub fn lex_chars(chars: Chars<'src>) -> TokenizeResult<'src> {
        let me = Self::new(None);

        let mut tokens = Vec::new();

        me.lex_into(chars, &mut tokens)?;

        Ok(tokens)
    }

    pub fn lex_into(
        self,
        chars: Chars<'src>,
        tokens: &mut Vec<Token<'src>>,
    ) -> Result<(), TokenizeError<'src>> {
        let mut chars = chars.peekable();
        let mut current_loc = self.current_loc;
        loop {
            if let Some(next_char) = chars.peek() {
                let single_chars = ['{', '}', ',', '[', ']', ':'];
                if single_chars.contains(next_char) {
                    let char = chars.next().unwrap();
                    let mut start_loc = current_loc.clone();
                    start_loc.len = 1;

                    current_loc.inc_ptr(false);

                    if char == '{' {
                        tokens.push(Token::ObjectStart(start_loc));
                    } else if char == '}' {
                        tokens.push(Token::ObjectEnd(start_loc));
                    } else if char == ',' {
                        tokens.push(Token::Comma(start_loc));
                    } else if char == '[' {
                        tokens.push(Token::ArrayStart(start_loc));
                    } else if char == ']' {
                        tokens.push(Token::ArrayEnd(start_loc));
                    } else if char == ':' {
                        tokens.push(Token::Colon(start_loc));
                    }
                } else if next_char.is_whitespace() {
                    tokens.push(Token::Whitespace(Self::lex_whitespace(
                        &mut current_loc,
                        &mut chars,
                    )?));
                } else {
                    tokens.push(Token::Literal(Self::lex_literal(
                        &mut current_loc,
                        &mut chars,
                    )?));
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    fn lex_whitespace(
        current_loc: &mut Span<'src>,
        chars: &mut Peekable<impl Iterator<Item = char>>,
    ) -> SpanResult<'src> {
        let mut my_loc = current_loc.clone();

        loop {
            if let Some(char) = chars.peek() {
                if char.is_whitespace() {
                    if char != &0x20.into()
                        && char != &0x0A.into()
                        && char != &0x0D.into()
                        && char != &0x09.into()
                    {
                        return Err(TokenizeError::IllegalWhitespace(Self::into_err_span(
                            &my_loc,
                        )));
                    }
                } else {
                    break;
                }

                my_loc.len += 1;
                current_loc.inc_ptr(char == &'\n');
                chars.next();
            } else {
                break;
            }
        }
        Ok(my_loc)
    }

    fn lex_literal(
        current_loc: &mut Span<'src>,
        chars: &mut Peekable<impl Iterator<Item = char>>,
    ) -> LiteralResult<'src> {
        if let Some(char) = chars.peek() {
            if char.is_numeric() || char == &'-' {
                let (span, string) = Self::lex_number(current_loc, chars)?;
                Ok(Literal::Number(span, string))
            } else if char == &'"' {
                let (span, string) = Self::lex_string(current_loc, chars)?;
                Ok(Literal::String(span, string))
            } else if let Some(span) = Self::lex_word_literal(current_loc, chars) {
                Ok(span)
            } else {
                Err(InvalidLiteral(current_loc.clone()))
            }
        } else {
            Err(EOF(current_loc.clone()))
        }
    }

    fn into_err_span(span: &Span<'src>) -> Span<'src> {
        let mut clone = span.clone();
        clone.len += 1;
        clone
    }

    fn lex_number(
        current_loc: &mut Span<'src>,
        chars: &mut Peekable<impl Iterator<Item = char>>,
    ) -> Result<(Span<'src>, String), TokenizeError<'src>> {
        let mut start_loc = current_loc.clone();
        let mut number = String::new();
        while let Some(char) = chars.peek() {
            if char.is_numeric() || matches!(char, '-' | 'e' | 'E' | '.' | '+') {
                number.push(*char);
                start_loc.len += 1;
                chars.next();
                current_loc.inc_ptr(false);
            } else {
                break;
            }
        }
        Ok((start_loc, number))
    }

    fn lex_string(
        current_loc: &mut Span<'src>,
        chars: &mut Peekable<impl Iterator<Item = char>>,
    ) -> Result<(Span<'src>, String), TokenizeError<'src>> {
        let mut start_loc = current_loc.clone();
        let mut string = String::new();

        let mut inc = || {
            start_loc.len += 1;
            current_loc.inc_ptr(false);
        };

        let mut in_string = false;
        loop {
            if let Some(char) = chars.peek() {
                if !in_string && char == &'"' {
                    chars.next();
                    inc();
                    in_string = true;
                } else if char == &'\\' {
                    string.push(*char);
                    chars.next();
                    inc();

                    let next_char = chars.peek();
                    if let Some(next_char) = next_char {
                        if next_char == &'\\'
                            || next_char == &'/'
                            || next_char == &'b'
                            || next_char == &'f'
                            || next_char == &'n'
                            || next_char == &'r'
                            || next_char == &'t'
                            || next_char == &'"'
                        {
                            string.push(*next_char);
                            inc();
                            chars.next();
                        } else if next_char == &'u' {
                            string.push(*next_char);
                            chars.next();
                            inc();
                            let hex_chars: Vec<char> = chars
                                .take(4)
                                .filter(|char| {
                                    char.is_numeric()
                                        || (char >= &'a' && char <= &'f')
                                        || (char >= &'A' && char <= &'F')
                                })
                                .collect();

                            if hex_chars.len() == 4 {
                                for c in hex_chars {
                                    string.push(c);
                                    inc();
                                }
                            } else {
                                return Err(InvalidEscape(Self::into_err_span(current_loc)));
                            }
                        } else {
                            return Err(InvalidEscape(Self::into_err_span(current_loc)));
                        }
                    } else {
                        return Err(InvalidEscape(Self::into_err_span(current_loc)));
                    }
                } else if in_string && char == &'"' {
                    chars.next();
                    inc();
                    return Ok((start_loc, string));
                } else if char <= &'\n' {
                    return Err(NewlineInString(Self::into_err_span(current_loc)));
                } else {
                    string.push(*char);
                    chars.next();
                    inc();
                }
            } else {
                return Err(UnterminatedString(Self::into_err_span(current_loc)));
            }
        }
    }

    fn lex_word_literal(
        current_loc: &mut Span<'src>,
        chars: &mut Peekable<impl Iterator<Item = char>>,
    ) -> Option<Literal<'src>> {
        let mut start_loc = current_loc.clone();
        let start_char = chars.peek().cloned();

        let test = |expected: &str| {
            let length = expected.len();
            let string: String = chars.take(length).collect();
            if string == expected {
                for _ in 0..length {
                    current_loc.inc_ptr(false);
                }
                start_loc.len = length;
                Some(start_loc)
            } else {
                None
            }
        };

        if let Some('n') = start_char {
            test("null").map(Literal::Null)
        } else if let Some('t') = start_char {
            test("true").map(Literal::True)
        } else if let Some('f') = start_char {
            test("false").map(Literal::False)
        } else {
            None
        }
    }
}
