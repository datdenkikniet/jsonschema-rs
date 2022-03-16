use std::{collections::HashMap, str::FromStr};

use crate::{
    json::{Json, Key},
    schema::uri::Uri,
};

use super::{
    keywords::{Contains, Enum, Items, LogicApplier, PrefixItems, PrimitiveType, Type},
    uri::UriParseError,
    JsonSchema, RootSchema,
};

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
    InvalidType,
}

impl<T> Into<Result<T, SchemaParseError>> for SchemaParseError {
    fn into(self) -> Result<T, SchemaParseError> {
        Err(self)
    }
}

macro_rules! parse_logic_kw {
    ($name: ident, $logic_type: expr) => {
        fn $name<'input>(
            key: Key,
            input: &'input Json,
        ) -> Result<RootSchema<'input>, SchemaParseError> {
            let array = match input {
                Json::Array(array) => {
                    if array.len() > 0 {
                        array
                    } else {
                        return SchemaParseError {
                            key,
                            kind: SchemaParseErrorKind::ArrayEmpty,
                        }
                        .into();
                    }
                }
                _ => {
                    return SchemaParseError {
                        key,
                        kind: SchemaParseErrorKind::NotArray,
                    }
                    .into();
                }
            };

            let mut all_ofs = Vec::new();
            for i in 0..array.len() {
                let entry = &array[i];
                all_ofs.push(Self::parse_json_schema_rec(key.copy_of(), entry)?);
            }

            Ok($logic_type(all_ofs).into())
        }
    };
}

#[derive(Debug, Clone)]
pub struct Parser;

impl Parser {
    pub fn parse_json_schema<'input>(
        input: &'input Json,
    ) -> Result<JsonSchema<'input>, SchemaParseError> {
        let key = Key::default();
        Self::parse_json_schema_rec(key, input)
    }

    fn parse_json_schema_rec<'input>(
        key: Key,
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
            Json::Object(object) => Self::parse_schema_object(key, object),
        }
    }

    fn parse_schema_object(
        key: Key,
        object: &HashMap<String, Json>,
    ) -> Result<JsonSchema, SchemaParseError> {
        let vocabulary = Self::parse_vocabulary(key.copy_of(), object.iter())?;
        let defs = Self::parse_defs(key.copy_of(), object)?;
        let id = Self::parse_id(key.copy_of(), object)?;
        let (other_schemas, unknowns) = Self::parse_root_schemas(
            key,
            object
                .iter()
                .filter(|(k, _v)| k != &"$vocabulary" && k != &"$defs" && k != &"$id"),
        )?;

        Ok(JsonSchema::new(
            id,
            vocabulary,
            defs,
            other_schemas,
            unknowns,
        ))
    }

    pub fn parse_root_schemas<'input, T>(
        key: Key,
        input: T,
    ) -> Result<(Vec<RootSchema<'input>>, HashMap<String, &'input Json>), SchemaParseError>
    where
        T: Iterator<Item = (&'input String, &'input Json)>,
    {
        let mut schemas = Vec::new();
        let mut unknowns = HashMap::new();

        for (k, v) in input {
            let key = key.copy_of().push_str(&k);
            let value = match k.as_str() {
                "allOf" => Self::parse_all_of(key, v)?,
                "anyOf" => Self::parse_any_of(key, v)?,
                "oneOf" => Self::parse_one_of(key, v)?,
                "not" => Self::parse_not(key, v)?,
                "enum" => Self::parse_enum(key, v)?,
                "type" => Self::parse_type(key, v)?,
                "items" => Self::parse_items(key, v)?,
                "prefixItems" => Self::parse_prefix_items(key, v)?,
                "contains" => Self::parse_contains(key, v)?,
                _ => {
                    unknowns.insert(k.clone(), v);
                    continue;
                }
            };
            schemas.push(value);
        }

        Ok((schemas, unknowns))
    }

    fn parse_vocabulary<'input, T>(
        key: Key,
        mut object: T,
    ) -> Result<Option<HashMap<Uri, bool>>, SchemaParseError>
    where
        T: Iterator<Item = (&'input String, &'input Json)>,
    {
        const VOCABULARY: &str = "$vocabulary";

        let vocab_key = key.push_str(VOCABULARY);

        let vocab_input = match object.find(|(k, _v)| k == &"$vocabulary") {
            Some((_k, Json::Object(vocab_input))) => vocab_input,
            None => return Ok(None),
            Some((k, _v)) => {
                return SchemaParseError {
                    key: vocab_key.push_str(k),
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
                        key: vocab_key.push_str(k),
                        kind: SchemaParseErrorKind::VocabularyNotBool,
                    }
                    .into();
                }
            };

            let uri = match Uri::from_string(k.clone()) {
                Ok(val) => val,
                Err(e) => {
                    return SchemaParseError {
                        key: vocab_key.push_str(k),
                        kind: SchemaParseErrorKind::InvalidUri(e),
                    }
                    .into();
                }
            };

            vocabulary.insert(uri, *required);
        }
        Ok(Some(vocabulary))
    }

    fn parse_defs(
        key: Key,
        object: &HashMap<String, Json>,
    ) -> Result<Option<HashMap<String, JsonSchema>>, SchemaParseError> {
        const DEFS: &str = "$defs";

        let defs_key = key.push_str(DEFS);

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
            let schema = match Self::parse_json_schema_rec(defs_key.copy_of().push_str(k), v) {
                Ok(schema) => schema,
                Err(mut e) => {
                    let key = defs_key.copy_of().push_str(&k);
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

    fn parse_not(key: Key, input: &Json) -> Result<RootSchema, SchemaParseError> {
        let schema = Self::parse_json_schema_rec(key, input)?;
        Ok(LogicApplier::Not(schema).into())
    }

    fn parse_enum(key: Key, input: &Json) -> Result<RootSchema, SchemaParseError> {
        let values = match input {
            Json::Array(values) => values,
            _ => {
                return SchemaParseError {
                    key,
                    kind: SchemaParseErrorKind::NotArray,
                }
                .into();
            }
        };
        Ok(Enum::new(values.iter().collect()).into())
    }

    fn parse_type(key: Key, input: &Json) -> Result<RootSchema, SchemaParseError> {
        let types = match input {
            Json::Array(values) => {
                let mut types = Vec::new();
                let mut i = 0;
                for value in values {
                    if let Json::String(value) = value {
                        if let Ok(ty) = PrimitiveType::from_str(value) {
                            types.push(ty)
                        } else {
                            return SchemaParseError {
                                key: key.push_idx(i),
                                kind: SchemaParseErrorKind::InvalidType,
                            }
                            .into();
                        }
                    } else {
                        return SchemaParseError {
                            key: key.push_idx(i),
                            kind: SchemaParseErrorKind::InvalidType,
                        }
                        .into();
                    }
                    i += 1;
                }
                types
            }
            Json::String(value_str) => {
                if let Ok(ty) = PrimitiveType::from_str(value_str) {
                    vec![ty]
                } else {
                    return Err(SchemaParseError {
                        key,
                        kind: SchemaParseErrorKind::InvalidType,
                    });
                }
            }
            _ => {
                return Err(SchemaParseError {
                    key,
                    kind: SchemaParseErrorKind::InvalidType,
                });
            }
        };
        Ok(Type::new(types).into())
    }

    fn parse_items(key: Key, input: &Json) -> Result<RootSchema, SchemaParseError> {
        let schema = Self::parse_json_schema_rec(key.copy_of(), input)?;
        Ok(RootSchema::Items(Items::new(schema)))
    }

    fn parse_prefix_items(key: Key, input: &Json) -> Result<RootSchema, SchemaParseError> {
        let array = match input {
            Json::Array(array) => array,
            _ => {
                return SchemaParseError {
                    key: key,
                    kind: SchemaParseErrorKind::InvalidType,
                }
                .into();
            }
        };

        let mut schemas = Vec::new();

        for i in 0..array.len() {
            let entry = &array[i];
            let schema = Self::parse_json_schema_rec(key.copy_of().push_idx(i), entry)?;
            schemas.push(schema);
        }

        Ok(RootSchema::PrefixItems(PrefixItems::new(schemas)))
    }

    fn parse_contains(key: Key, input: &Json) -> Result<RootSchema, SchemaParseError> {
        let schema = Self::parse_json_schema_rec(key, input)?;
        Ok(RootSchema::Contains(Contains::new(schema)))
    }

    fn parse_id(key: Key, input: &HashMap<String, Json>) -> Result<Option<Uri>, SchemaParseError> {
        let key = key.push_str("$id");
        let string = match input.get("$id") {
            Some(Json::String(id)) => id,
            None => return Ok(None),
            _ => {
                return SchemaParseError {
                    key: key.copy_of().push_str("$id"),
                    kind: SchemaParseErrorKind::InvalidType,
                }
                .into()
            }
        };
        match Uri::from_str(string) {
            Ok(val) => Ok(Some(val)),
            Err(e) => SchemaParseError {
                key,
                kind: SchemaParseErrorKind::InvalidUri(e),
            }
            .into(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{
        json::Json,
        schema::{parser::Parser, uri::Uri, JsonSchema},
    };

    macro_rules! test {
        ($name: ident, $input: literal, $output: expr) => {
            let input = &Json::from_string($input).unwrap();
            let result = Parser::parse_json_schema(input).unwrap();
            assert_eq!(result, $output);
        };
    }

    #[test]
    fn all() {
        let input = Json::from_string(
            r#"
            {
                "items": "items",
                "prefixItems": ["prefix", "items"],
                "contains": "contains"
            }
        "#,
        )
        .unwrap();

        let schema = Parser::parse_json_schema(&input);
        panic!("{:#?}", schema.unwrap());
    }

    #[test]
    fn vocabulary() {
        let mut vocabs = HashMap::new();
        vocabs.insert(Uri::from_str("some_vocab").unwrap(), true);
        vocabs.insert(Uri::from_str("some_other_vocab").unwrap(), false);

        let schema = JsonSchema {
            id: None,
            vocabulary: Some(vocabs),
            defs: None,
            schemas: Vec::new(),
            unknowns: HashMap::new(),
        };

        test!(
            vocabulary,
            r#"
            {
                "$vocabulary": {
                    "some_vocab": true,
                    "some_other_vocab": false
                }
            }
            "#,
            schema
        );
    }
}
