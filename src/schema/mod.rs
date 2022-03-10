mod keywords;

use crate::json::{Json, Key};

use self::keywords::{
    annotations::{LogicError, PropertyError, TypeError},
    LogicApplier, Property, Type,
};

trait JsonSchemaValidator {
    fn validate_json<'schema>(
        &'schema self,
        input: &Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool;
}

trait AnnotationValue {
    fn is_error(&self) -> bool;
}

impl AnnotationValue for bool {
    fn is_error(&self) -> bool {
        !self
    }
}

impl<T> AnnotationValue for Option<T>
where
    T: AnnotationValue,
{
    fn is_error(&self) -> bool {
        match self {
            Some(value) => value.is_error(),
            None => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Annotation<'schema> {
    LogicError(LogicError<'schema>),
    PropertyError(PropertyError<'schema>),
    TypeError(TypeError),
    Unequal {
        schema: &'schema JsonSchema<'schema>,
        key: Key,
    },
}

#[derive(Debug, Clone)]
pub enum JsonSchema<'schema> {
    Primitive(Json),
    Logic(LogicApplier<'schema>),
    Property(Property<'schema>),
    Type(Type),
}

impl<'schema> JsonSchema<'schema> {
    fn validate_json<'input>(
        &'schema self,
        input: &'input Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool {
        let mut success = true;
        let key = Key::default();
        match self {
            JsonSchema::Primitive(primitive) => {
                if input != primitive {
                    success = false;
                    annotations.push(
                        Annotation::Unequal {
                            schema: self,
                            key: key.copy_of(),
                        }
                        .into(),
                    );
                }
            }
            JsonSchema::Logic(logic) => {
                if !logic.validate_json(input, annotations) {
                    success = false;
                }
            }
            JsonSchema::Property(properties) => {
                if !properties.validate_json(input, annotations) {
                    success = false;
                }
            }
            JsonSchema::Type(ty) => {
                if !ty.validate_json(input, annotations) {
                    success = false;
                }
            }
        }
        success
    }
}

impl<'schema> JsonSchema<'schema> {
    pub fn validate<'a>(&'a self, input: &'a Json) -> Vec<Annotation> {
        let mut annotations = Vec::new();
        self.validate_json(input, &mut annotations);
        annotations
    }
}

impl<'schema> From<Json> for JsonSchema<'schema> {
    fn from(input: Json) -> Self {
        Self::Primitive(input)
    }
}

impl<'schema> From<&str> for JsonSchema<'schema> {
    fn from(input: &str) -> Self {
        <&str as Into<Json>>::into(input).into()
    }
}
