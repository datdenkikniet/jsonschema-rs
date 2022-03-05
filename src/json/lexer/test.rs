use crate::json::lexer::{Literal, Span, Token, TokenizeError};

macro_rules! test_single {
    ($name: ident, $input: expr, $output: expr, succeeds) => {
        #[test]
        fn $name() {
            let tokens = Token::from_str($input).unwrap();
            assert_eq!(tokens, vec![$output]);
        }
    };
    ($name: ident, $input: expr, $output: expr, fails) => {
        #[test]
        fn $name() {
            if let Err(error) = Token::from_str($input) {
                assert_eq!(error, $output)
            } else {
                panic!(
                    "Function {} is supposed to fail, but succeeded",
                    stringify!($name)
                );
            }
        }
    };
}

const STRING: &str = "\"Hello\"";
test_single!(
    string,
    STRING,
    Token::Literal(Literal::String(Span::new(STRING, 0, 0, 0, STRING.len()))),
    succeeds
);
