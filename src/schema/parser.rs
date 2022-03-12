use std::collections::HashMap;

use crate::{
    json::{Json, Key, KeyPart},
    schema::uri::Uri,
};

use super::{keywords::LogicApplier, uri::UriParseError, JsonSchema, RootSchema};

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
    NotObject,
    NotArray,
    ArrayEmpty,
}

impl<T> Into<Result<T, SchemaParseError>> for SchemaParseError {
    fn into(self) -> Result<T, SchemaParseError> {
        Err(self)
    }
}

macro_rules! parse_logic_kw {
    ($name: ident, $logic_type: expr) => {
        fn $name<'input>(input: &'input Json) -> Result<RootSchema<'input>, SchemaParseError> {
            let array = match input {
                Json::Array(array) => {
                    if array.len() > 0 {
                        array
                    } else {
                        return SchemaParseError {
                            key: Key::default(),
                            kind: SchemaParseErrorKind::ArrayEmpty,
                        }
                        .into();
                    }
                }
                _ => {
                    return SchemaParseError {
                        key: Key::default(),
                        kind: SchemaParseErrorKind::NotArray,
                    }
                    .into();
                }
            };

            let mut all_ofs = Vec::new();
            for i in 0..array.len() {
                let entry = &array[i];
                all_ofs.push(Self::parse_json_schema(entry)?);
            }

            Ok(RootSchema::Logic($logic_type(all_ofs)))
        }
    };
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
            | Json::Null => Ok(JsonSchema::with_root_schemas(vec![RootSchema::Primitive(
                &input,
            )])),
            Json::Object(object) => Self::parse_schema_object(object),
        }
    }

    fn parse_schema_object<'input>(
        object: &'input HashMap<String, Json>,
    ) -> Result<JsonSchema<'input>, SchemaParseError> {
        let vocabulary = Self::parse_vocabulary(object)?;
        let defs = Self::parse_defs(object)?;
        let (other_schemas, unknowns) = Self::parse_root_schemas(object)?;

        Ok(JsonSchema::new(
            None,
            vocabulary,
            defs,
            other_schemas,
            unknowns,
        ))
    }

    pub fn parse_root_schemas<'input>(
        input: &'input HashMap<String, Json>,
    ) -> Result<(Vec<RootSchema<'input>>, HashMap<String, &Json>), SchemaParseError> {
        let mut schemas = Vec::new();
        let mut unknowns = HashMap::new();

        for (k, v) in input {
            let value = match k.as_str() {
                "$defs" | "$vocabulary" | "$id" => continue,
                "allOf" => Self::parse_all_of(v)?,
                "anyOf" => Self::parse_any_of(v)?,
                "oneOf" => Self::parse_one_of(v)?,
                "not" => Self::parse_not(v)?,
                _ => {
                    unknowns.insert(k.clone(), v);
                    continue;
                }
            };
            schemas.push(value);
        }

        Ok((schemas, unknowns))
    }

    fn parse_vocabulary<'input>(
        object: &'input HashMap<String, Json>,
    ) -> Result<Option<HashMap<Uri, bool>>, SchemaParseError> {
        const VOCABULARY: &str = "$vocabulary";

        let vocab_key = Key::new(vec![KeyPart::Identifier(VOCABULARY.to_string())]);

        let vocab_input = match object.get("$vocabulary") {
            Some(Json::Object(vocab_input)) => vocab_input,
            None => return Ok(None),
            Some(_) => {
                return SchemaParseError {
                    key: vocab_key.copy_of(),
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
                    return SchemaParseError {
                        key: vocab_key.push(KeyPart::Identifier(k.clone())),
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
    ) -> Result<Option<HashMap<String, JsonSchema<'input>>>, SchemaParseError> {
        const DEFS: &str = "$defs";

        let defs_key = Key::new(vec![KeyPart::Identifier(DEFS.to_string())]);

        let defs_input = match object.get(DEFS) {
            Some(Json::Object(object)) => object,
            None => return Ok(None),
            Some(_) => {
                return SchemaParseError {
                    key: defs_key,
                    kind: SchemaParseErrorKind::NotObject,
                }
                .into();
            }
        };

        let mut schemas = HashMap::new();

        for (k, v) in defs_input {
            let schema = match Self::parse_json_schema(v) {
                Ok(schema) => schema,
                Err(mut e) => {
                    let key = defs_key.copy_of().push(KeyPart::Identifier(k.clone()));
                    e.key = key;
                    return Err(e);
                }
            };
            schemas.insert(k.clone(), schema);
        }

        Ok(Some(schemas))
    }

    parse_logic_kw!(parse_all_of, LogicApplier::AllOf);
    parse_logic_kw!(parse_any_of, LogicApplier::AnyOf);
    parse_logic_kw!(parse_one_of, LogicApplier::OneOf);

    fn parse_not<'input>(input: &'input Json) -> Result<RootSchema<'input>, SchemaParseError> {
        let schema = Self::parse_json_schema(input)?;
        Ok(RootSchema::Logic(LogicApplier::Not(schema)))
    }
}

#[test]
fn parse_correct() {
    let input = &Json::from_string(
        r#"
        {
            "$vocabulary": {
                "some_vocab": true,
                "some_other_vocab": false
            }
        }
    "#,
    )
    .unwrap();

    let result = Parser::parse_json_schema(input);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn all_of() {
    let input = &Json::from_string(
        r#"
            {
                "allOf": ["this", "and", "that"],
                "not": "hello"
            }
            "#,
    )
    .unwrap();

    let result = Parser::parse_json_schema(input).unwrap();
    panic!("{:#?}", result);
}

#[test]
fn defs() {
    let input = &Json::from_string(
        r#"
        {
            "$defs": {
                "nested": {
                    "not": "nested"
                }
            }
        }
        "#,
    )
    .unwrap();

    let result = Parser::parse_json_schema(input).unwrap();
    panic!("{:#?}", result);
}
