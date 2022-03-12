use std::collections::HashMap;

use crate::{
    json::{Json, Key, KeyPart},
    schema::uri::Uri,
};

use super::{uri::UriParseError, JsonSchema, RootSchema};

#[derive(Debug, Clone)]
pub struct SchemaParseError {
    pub key: Key,
    pub kind: SchemaParseErrorKind,
}

#[derive(Debug, Clone)]
pub enum SchemaParseErrorKind {
    IllegalVocabularyType,
    InvalidUri(UriParseError),
    VocabularyNotBool,
    IllegalDefsType,
}

impl<T> Into<Result<T, SchemaParseError>> for SchemaParseError {
    fn into(self) -> Result<T, SchemaParseError> {
        Err(self)
    }
}

#[derive(Debug, Clone)]
pub struct Parser;

impl Parser {
    pub fn parse_json_schema<'input>(
        input: &'input Json,
    ) -> Result<JsonSchema<'input>, SchemaParseError> {
        match input {
            Json::Array(_)
            | Json::Number { .. }
            | Json::String(_)
            | Json::Boolean(_)
            | Json::Null => Ok(JsonSchema::with_root_schema(RootSchema::Primitive(&input))),
            Json::Object(object) => Self::parse_schema_object(object),
        }
    }

    fn parse_schema_object<'input>(
        object: &'input HashMap<String, Json>,
    ) -> Result<JsonSchema<'input>, SchemaParseError> {
        let vocabulary = Self::parse_vocabulary(object)?;
        let defs = Self::parse_defs(object)?;

        Ok(JsonSchema::new(None, vocabulary, defs, None))
    }

    pub fn parse_root_schema<'input>(
        _input: &HashMap<String, Json>,
    ) -> Result<RootSchema<'input>, SchemaParseError> {
        todo!()
    }

    fn parse_vocabulary<'input>(
        object: &'input HashMap<String, Json>,
    ) -> Result<Option<HashMap<Uri, bool>>, SchemaParseError> {
        const VOCABULARY: &str = "$vocabulary";

        let mut vocab_key = Key::new(vec![KeyPart::Identifier(VOCABULARY.to_string())]);

        let vocab_input = match object.get("$vocabulary") {
            Some(Json::Object(vocab_input)) => vocab_input,
            None => return Ok(None),
            Some(_) => {
                return SchemaParseError {
                    key: vocab_key,
                    kind: SchemaParseErrorKind::IllegalVocabularyType,
                }
                .into()
            }
        };

        let mut vocabulary = HashMap::new();
        for (k, v) in vocab_input {
            let required = match v {
                Json::Boolean(req) => req,
                _ => {
                    vocab_key.push(KeyPart::Identifier(k.clone()));
                    return SchemaParseError {
                        key: vocab_key,
                        kind: SchemaParseErrorKind::VocabularyNotBool,
                    }
                    .into();
                }
            };

            let uri = match Uri::from_string(k.clone()) {
                Ok(val) => val,
                Err(e) => {
                    return SchemaParseError {
                        key: vocab_key.copy_of(),
                        kind: SchemaParseErrorKind::InvalidUri(e),
                    }
                    .into();
                }
            };

            vocabulary.insert(uri, *required);
        }
        Ok(Some(vocabulary))
    }

    fn parse_defs<'input>(
        object: &'input HashMap<String, Json>,
    ) -> Result<Option<HashMap<String, RootSchema<'input>>>, SchemaParseError> {
        const DEFS: &str = "$defs";

        let mut defs_key = Key::new(vec![KeyPart::Identifier(DEFS.to_string())]);

        let defs_input = match object.get(DEFS) {
            Some(Json::Object(object)) => object,
            None => return Ok(None),
            Some(_) => {
                return SchemaParseError {
                    key: defs_key,
                    kind: SchemaParseErrorKind::IllegalDefsType,
                }
                .into();
            }
        };

        let mut schemas = HashMap::new();

        for (k, v) in defs_input {
            let maybe_schema = match v {
                Json::Object(schema) => schema,
                _ => {
                    defs_key.push(KeyPart::Identifier(k.clone()));
                    return SchemaParseError {
                        key: defs_key,
                        kind: SchemaParseErrorKind::IllegalDefsType,
                    }
                    .into();
                }
            };

            let schema = Self::parse_root_schema(maybe_schema)?;
            schemas.insert(k.clone(), schema);
        }

        Ok(Some(schemas))
    }
}

#[test]
fn parse_correct() {
    let input = Json::from_string(
        r#"
        {
            "$vocabulary": {
                "some_vocab": true,
                "some_other_vocab": true
            }
        }
    "#,
    )
    .unwrap();

    let result = Parser::parse_json_schema(&input);
    assert!(result.is_ok(), "{:?}", result);
}
