mod lexer;
pub use lexer::Lexer;

use std::collections::HashMap;

use crate::json::lexer::Token;

use self::lexer::{Literal, Span, TokenizeError};

#[derive(Debug, Clone)]
pub enum ParseError<'src> {
    TokenizeError(TokenizeError<'src>),
    InvalidNumber(Span<'src>),
    UnclosedArray(Span<'src>),
    UnclosedObject(Span<'src>),
    IllegalArray(Span<'src>),
    IllegalObject(Span<'src>),
    LeftOverTokens(Span<'src>),
    InvalidKeyType(Span<'src>),
    ColonExpected(Span<'src>),
    IllegalLeadingZero(Span<'src>),
    ExtraColon(Span<'src>),
    ExtraComma(Span<'src>),
    UnopenedObject(Span<'src>),
    UnopenedArray(Span<'src>),
    NoMoreTokens,
}

impl<'src> From<TokenizeError<'src>> for ParseError<'src> {
    fn from(tok: TokenizeError<'src>) -> Self {
        Self::TokenizeError(tok)
    }
}

impl<'src> ParseError<'src> {
    pub fn span(&self) -> Option<&Span<'src>> {
        match self {
            Self::InvalidNumber(span)
            | Self::UnclosedArray(span)
            | Self::UnclosedObject(span)
            | Self::IllegalArray(span)
            | Self::IllegalObject(span)
            | Self::LeftOverTokens(span)
            | Self::InvalidKeyType(span)
            | Self::ColonExpected(span)
            | Self::ExtraColon(span)
            | Self::ExtraComma(span)
            | Self::UnopenedObject(span)
            | Self::UnopenedArray(span)
            | Self::IllegalLeadingZero(span) => Some(span),
            Self::TokenizeError(error) => Some(error.span()),
            Self::NoMoreTokens => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Object(HashMap<String, Json>),
    Array(Vec<Json>),
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
}

impl Json {
    pub fn from_tokens<'src>(tokens_in: &'src [Token]) -> Result<Option<Self>, ParseError<'src>> {
        let tokens = tokens_in
            .iter()
            .filter(|tok| !matches!(tok, Token::Whitespace(_)));

        let (result, mut iter) = Self::parse_first_token(tokens)?;
        if let Some(token) = iter.next() {
            Err(ParseError::LeftOverTokens(token.span()))
        } else {
            Ok(result)
        }
    }

    fn parse_first_token<'src, T>(mut tokens: T) -> Result<(Option<Self>, T), ParseError<'src>>
    where
        T: Iterator<Item = &'src Token<'src>> + Clone,
    {
        let value = if let Some(token) = tokens.next() {
            match token {
                Token::Literal(Literal::Number(span, value)) => {
                    Some(Self::parse_number(span, value)?)
                }
                Token::Literal(Literal::String(_, value)) => Some(Self::String(value.clone())),
                Token::Literal(Literal::Null(_)) => Some(Self::Null),
                Token::Literal(Literal::False(_)) => Some(Self::Boolean(false)),
                Token::Literal(Literal::True(_)) => Some(Self::Boolean(true)),
                Token::Whitespace(_) => None,
                Token::ArrayStart(start) => {
                    let res = Self::parse_array(start, tokens)?;
                    return Ok(res);
                }
                Token::ObjectStart(start) => {
                    let res = Self::parse_object(start, tokens)?;
                    return Ok(res);
                }
                Token::Colon(span) => return Err(ParseError::ExtraColon(span.clone())),
                Token::Comma(_span) => None,
                Token::ObjectEnd(span) => return Err(ParseError::UnopenedObject(span.clone())),
                Token::ArrayEnd(span) => return Err(ParseError::UnopenedArray(span.clone())),
            }
        } else {
            return Err(ParseError::NoMoreTokens);
        };

        Ok((value, tokens))
    }

    fn parse_number<'src>(input: &Span<'src>, value: &String) -> Result<Json, ParseError<'src>> {
        let (integer, rest, has_fraction, has_exponent) = {
            let (split_char, has_fraction, has_exponent) = if value.contains(".") {
                (Some("."), true, value.contains("E") || value.contains("e"))
            } else if value.contains("e") {
                (Some("e"), false, true)
            } else if value.contains("E") {
                (Some("E"), false, true)
            } else {
                (None, false, false)
            };

            if let Some(split_char) = split_char {
                let mut parts = value.split(split_char);
                let res = (parts.next(), parts.next(), has_fraction, has_exponent);
                if parts.next().is_none() {
                    res
                } else {
                    return Err(ParseError::InvalidNumber(input.clone()));
                }
            } else {
                (Some(value.as_str()), None, has_fraction, has_exponent)
            }
        };

        let (fraction, exponent) = if let Some(rest) = rest {
            let split_char = if rest.contains("e") {
                Some("e")
            } else if rest.contains("E") {
                Some("E")
            } else {
                None
            };

            if let Some(split_char) = split_char {
                let mut parts = rest.split(split_char);
                let res = if has_fraction {
                    (parts.next(), parts.next())
                } else {
                    (None, parts.next())
                };

                if parts.next().is_none() {
                    res
                } else {
                    return Err(ParseError::InvalidNumber(input.clone()));
                }
            } else if has_fraction {
                (Some(rest), None)
            } else if has_exponent {
                (None, Some(rest))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        let test_input = |mut value: &'_ str, allow_leading_zero: bool| {
            let negative = if value.starts_with("-") {
                value = &value[1..];
                -1.0
            } else {
                1.0
            };

            if value.len() == 0 {
                return Err(ParseError::InvalidNumber(input.clone()));
            }

            let leading_zero = value.chars().nth(0) == Some('0');

            let number = if let Ok(val) = i64::from_str_radix(value, 10) {
                val
            } else {
                return Err(ParseError::InvalidNumber(input.clone()));
            };

            if number != 0 && leading_zero && !allow_leading_zero {
                return Err(ParseError::IllegalLeadingZero(input.clone()));
            }

            Ok(negative * number as f64)
        };

        let mut final_val = if let Some(integer) = integer {
            test_input(integer, false)?
        } else {
            return Err(ParseError::InvalidNumber(input.clone()));
        };

        if let Some(fraction) = fraction {
            let value = test_input(fraction, true)?;

            if value < 0.0 {
                return Err(ParseError::InvalidNumber(input.clone()));
            } else {
                let value = value * (10.0f64.powf(-1.0 * (fraction.len() as f64)));
                if final_val < 0.0 {
                    final_val -= value;
                } else {
                    final_val += value;
                }
            }
        }

        if let Some(exponent) = exponent {
            let value = 10.0f64.powf(test_input(exponent, false)?);
            final_val = final_val * value;
        };

        Ok(Json::Number(final_val))
    }

    fn parse_object<'src, T>(
        start: &Span<'src>,
        mut object_tok: T,
    ) -> Result<(Option<Json>, T), ParseError<'src>>
    where
        T: Iterator<Item = &'src Token<'src>> + Clone,
    {
        let mut data = HashMap::new();
        loop {
            let first_token = if let Some(tok) = object_tok.next() {
                tok
            } else {
                return Err(ParseError::UnclosedObject(start.clone()));
            };

            let name = {
                let possible_name = if matches!(first_token, Token::ObjectEnd(_)) {
                    break;
                } else if data.is_empty() {
                    first_token
                } else if !matches!(first_token, Token::Comma(_)) {
                    return Err(ParseError::IllegalObject(first_token.span()));
                } else {
                    if let Some(next) = object_tok.next() {
                        next
                    } else {
                        return Err(ParseError::IllegalObject(first_token.span()));
                    }
                };

                if let Token::Literal(Literal::String(_, name)) = possible_name {
                    name
                } else {
                    return Err(ParseError::InvalidKeyType(first_token.span()));
                }
            };

            if !matches!(object_tok.next(), Some(Token::Colon(_))) {
                return Err(ParseError::ColonExpected(object_tok.next().unwrap().span()));
            }

            let (parsed, non_cons_tokens) = Self::parse_first_token(object_tok.clone())?;
            object_tok = non_cons_tokens;

            if let Some(parsed) = parsed {
                data.insert(name.clone(), parsed);
            } else {
                return Err(ParseError::NoMoreTokens);
            }
        }

        Ok((Some(Self::Object(data)), object_tok))
    }

    fn parse_array<'src, T>(
        start: &Span<'src>,
        mut array_tokens: T,
    ) -> Result<(Option<Json>, T), ParseError<'src>>
    where
        T: Iterator<Item = &'src Token<'src>> + Clone,
    {
        let mut data = Vec::new();

        loop {
            if data.is_empty() {
                if let Some(tok) = array_tokens.clone().next() {
                    if matches!(tok, Token::ArrayEnd(_)) {
                        array_tokens.next();
                        break;
                    }
                } else {
                    return Err(ParseError::UnclosedArray(start.clone()));
                }
            } else {
                if let Some(tok) = array_tokens.next() {
                    if matches!(tok, Token::ArrayEnd(_)) {
                        break;
                    } else if !matches!(tok, Token::Comma(_)) {
                        return Err(ParseError::ExtraComma(tok.span().clone()));
                    }
                } else {
                    return Err(ParseError::UnclosedArray(start.clone()));
                }
            };

            let (parsed, non_cons_tokens) = Self::parse_first_token(array_tokens.clone())?;
            array_tokens = non_cons_tokens;

            if let Some(entry) = parsed {
                data.push(entry);
            } else {
                return Err(ParseError::IllegalArray(
                    array_tokens.next().unwrap().span().clone(),
                ));
            }
        }

        Ok((Some(Self::Array(data)), array_tokens))
    }

    pub fn to_string(&self) -> String {
        let mut string = String::new();

        self.to_string_rec(&mut string);

        string
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
            Json::Number(number) => buffer.push_str(format!("{}", *number).as_str()),
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
