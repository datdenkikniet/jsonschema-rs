pub mod keywords;

pub mod parser;

pub mod uri;

use std::collections::HashMap;

use crate::json::{Json, Key};

use keywords::{
    annotations::{EnumError, LogicError, PropertyError, TypeError},
    LogicApplier, Property, Type,
};

use self::uri::Uri;

trait JsonSchemaValidator {
    fn validate_json<'schema>(
        &'schema self,
        key_to_input: &mut Key,
        input: &Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool;
}

trait AnnotationValue {
    fn is_error(&self) -> bool;
}

#[derive(Debug, Clone)]
pub enum Annotation<'schema> {
    LogicError(LogicError<'schema>),
    PropertyError(PropertyError<'schema>),
    TypeError(TypeError),
    EnumError(EnumError),
    Unequal {
        schema: &'schema RootSchema<'schema>,
        key: Key,
    },
}

#[derive(Debug, Clone)]
pub struct JsonSchema<'schema> {
    id: Option<Uri>,
    vocabulary: Option<HashMap<Uri, bool>>,
    defs: Option<HashMap<String, RootSchema<'schema>>>,
    root_schema: Option<RootSchema<'schema>>,
}

impl<'schema> JsonSchema<'schema> {
    pub fn new(
        id: Option<Uri>,
        vocabulary: Option<HashMap<Uri, bool>>,
        defs: Option<HashMap<String, RootSchema<'schema>>>,
        root_schema: Option<RootSchema<'schema>>,
    ) -> Self {
        Self {
            id,
            vocabulary,
            defs,
            root_schema,
        }
    }

    pub fn with_root_schema(root_schema: RootSchema<'schema>) -> Self {
        Self {
            id: None,
            vocabulary: None,
            defs: None,
            root_schema: Some(root_schema),
        }
    }

    pub fn id(&self) -> &Option<Uri> {
        &self.id
    }

    pub fn vocabulary(&self) -> &Option<HashMap<Uri, bool>> {
        &self.vocabulary
    }

    pub fn defs(&self) -> &Option<HashMap<String, RootSchema>> {
        &self.defs
    }

    pub fn root_schema(&self) -> &Option<RootSchema> {
        &self.root_schema
    }
}

#[derive(Debug, Clone)]
pub enum RootSchema<'schema> {
    Primitive(&'schema Json),
    Logic(LogicApplier<'schema>),
    Properties(Vec<Property<'schema>>),
    Type(Type),
}

impl<'schema> RootSchema<'schema> {
    fn validate_json<'input>(
        &'schema self,
        key_to_input: &mut Key,
        input: &'input Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool {
        let mut success = true;
        match self {
            RootSchema::Primitive(primitive) => {
                if &input != primitive {
                    success = false;
                    annotations.push(
                        Annotation::Unequal {
                            schema: self,
                            key: key_to_input.copy_of(),
                        }
                        .into(),
                    );
                }
            }
            RootSchema::Logic(logic) => {
                if !logic.validate_json(key_to_input, input, annotations) {
                    success = false;
                }
            }
            RootSchema::Properties(properties) => {
                for property in properties {
                    if !property.validate_json(key_to_input, input, annotations) {
                        success = false;
                    }
                }
            }
            RootSchema::Type(ty) => {
                if !ty.validate_json(key_to_input, input, annotations) {
                    success = false;
                }
            }
        }

        success
    }
}

impl<'schema> RootSchema<'schema> {
    pub fn validate<'a>(&'a self, input: &'a Json) -> ValidationResult<'a> {
        let mut annotations = Vec::new();
        let key_to_input = &mut Key::default();
        let validation_success = self.validate_json(key_to_input, input, &mut annotations);
        ValidationResult {
            success: validation_success,
            annotations,
        }
    }
}

impl<'schema> From<&'schema Json> for RootSchema<'schema> {
    fn from(input: &'schema Json) -> Self {
        Self::Primitive(input)
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult<'schema> {
    pub success: bool,
    pub annotations: Vec<Annotation<'schema>>,
}

impl<'schema> ValidationResult<'schema> {
    pub fn success(&self) -> bool {
        self.success
    }
    pub fn annotations(&self) -> &Vec<Annotation> {
        &self.annotations
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        json::{Lexer, Parser},
        schema::{
            keywords::{Property, Type},
            RootSchema,
        },
    };

    #[test]
    fn big_test() {
        let input = r#"{"first_key": "value", "second_key": {"first_nested_key": 1e23, "second_nested_key": "123"}}"#;
        let tokens = &mut Vec::new();
        Lexer::new(Some(input))
            .lex_into(input.chars(), tokens)
            .unwrap();

        let input = Parser::parse_tokens(&tokens).unwrap().unwrap();

        let second_level = RootSchema::Properties(vec![
            Property::new(
                "first_nested_key",
                vec![&RootSchema::Type(Type::Number)],
                false,
            ),
            Property::new(
                "second_nested_key",
                vec![&RootSchema::Type(Type::String)],
                false,
            ),
        ]);

        let first_level = RootSchema::Properties(vec![
            Property::new("first_key", vec![&RootSchema::Type(Type::String)], false),
            Property::new("second_key", vec![&second_level], false),
        ]);

        let validation = first_level.validate(&input);

        assert!(validation.annotations().is_empty(), "{:?}", validation);
    }
}
