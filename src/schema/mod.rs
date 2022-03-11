pub mod keywords;

use crate::json::{Json, Key};

use keywords::{
    annotations::{EnumError, LogicError, PropertyError, TypeError},
    LogicApplier, Property, Type,
};

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
        schema: &'schema JsonSchema<'schema>,
        key: Key,
    },
}

#[derive(Debug, Clone)]
pub enum JsonSchema<'schema> {
    Primitive(Json),
    Logic(LogicApplier<'schema>),
    Properties(Vec<Property<'schema>>),
    Type(Type),
}

impl<'schema> JsonSchema<'schema> {
    fn validate_json<'input>(
        &'schema self,
        key_to_input: &mut Key,
        input: &'input Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool {
        let mut success = true;
        match self {
            JsonSchema::Primitive(primitive) => {
                if input != primitive {
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
            JsonSchema::Logic(logic) => {
                if !logic.validate_json(key_to_input, input, annotations) {
                    success = false;
                }
            }
            JsonSchema::Properties(properties) => {
                for property in properties {
                    if !property.validate_json(key_to_input, input, annotations) {
                        success = false;
                    }
                }
            }
            JsonSchema::Type(ty) => {
                if !ty.validate_json(key_to_input, input, annotations) {
                    success = false;
                }
            }
        }

        success
    }
}

impl<'schema> JsonSchema<'schema> {
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

impl<'schema> From<Json> for JsonSchema<'schema> {
    fn from(input: Json) -> Self {
        Self::Primitive(input)
    }
}

impl<'schema> From<&str> for JsonSchema<'schema> {
    fn from(input: &str) -> Self {
        Json::from(input).into()
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
            JsonSchema,
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

        let second_level = JsonSchema::Properties(vec![
            Property::new(
                "first_nested_key",
                vec![&JsonSchema::Type(Type::Number)],
                false,
            ),
            Property::new(
                "second_nested_key",
                vec![&JsonSchema::Type(Type::String)],
                false,
            ),
        ]);

        let first_level = JsonSchema::Properties(vec![
            Property::new("first_key", vec![&JsonSchema::Type(Type::String)], false),
            Property::new("second_key", vec![&second_level], false),
        ]);

        let validation = first_level.validate(&input);

        assert!(validation.annotations().is_empty(), "{:?}", validation);
    }
}
