use crate::{
    json::{Json, Key},
    schema::{Annotation, JsonSchemaValidator},
};

#[derive(Debug, Clone)]
pub enum TypeErrorKind {
    TypeMismatch,
    NotInteger,
}

#[derive(Debug, Clone)]
pub struct TypeError {
    pub key: Key,
    pub error: TypeErrorKind,
    pub actual: Type,
}

impl<'schema> Into<Annotation<'schema>> for TypeError {
    fn into(self) -> Annotation<'schema> {
        Annotation::TypeError(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    String,
    Number,
    Integer,
    Object,
    Array,
    Boolean,
    Null,
}

impl From<&Json> for Type {
    fn from(input: &Json) -> Self {
        match input {
            Json::Object(_) => Self::Object,
            Json::Array(_) => Self::Array,
            Json::Number { .. } => Self::Number,
            Json::String(_) => Self::String,
            Json::Boolean(_) => Self::Boolean,
            Json::Null => Self::Null,
        }
    }
}

impl JsonSchemaValidator for Type {
    fn validate_json<'schema>(
        &'schema self,
        input: &Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool {
        let key = Key::default();

        let error_kind = if let (Type::Integer, Json::Number { fraction, .. }) = (self, input) {
            if fraction.1 == 0 {
                None
            } else {
                Some(TypeErrorKind::NotInteger)
            }
        } else if self != &input.into() {
            Some(TypeErrorKind::TypeMismatch)
        } else {
            None
        };

        if let Some(type_error) = error_kind {
            annotations.push(
                TypeError {
                    key: key.copy_of(),
                    error: type_error,
                    actual: input.into(),
                }
                .into(),
            );
            false
        } else {
            true
        }
    }
}
