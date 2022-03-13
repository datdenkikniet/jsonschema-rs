use std::str::FromStr;

use crate::{
    json::{Json, Key},
    schema::{Annotation, JsonSchemaValidator},
};

#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    pub key: Key,
    pub actual: PrimitiveType,
}

impl<'schema> Into<Annotation<'schema>> for TypeError {
    fn into(self) -> Annotation<'schema> {
        Annotation::TypeError(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveType {
    String,
    Number,
    Integer,
    Object,
    Array,
    Boolean,
    Null,
}

impl From<&Json> for PrimitiveType {
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

impl FromStr for PrimitiveType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let me = match s {
            "null" => Self::Null,
            "boolean" => Self::Boolean,
            "object" => Self::Object,
            "array" => Self::Array,
            "number" => Self::Number,
            "string" => Self::String,
            "integer" => Self::Integer,
            _ => return Err(s.to_string()),
        };
        Ok(me)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    types: Vec<PrimitiveType>,
}

impl From<PrimitiveType> for Type {
    fn from(other: PrimitiveType) -> Self {
        Self { types: vec![other] }
    }
}

impl JsonSchemaValidator for Type {
    fn validate_json<'schema>(
        &'schema self,
        key_to_input: Key,
        input: &Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool {
        let mut found = false;
        for ty in &self.types {
            if ty == &PrimitiveType::from(input) {
                found = true;
                break;
            }
        }

        if !found {
            annotations.push(
                TypeError {
                    key: key_to_input.copy_of(),
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

impl Type {
    pub fn new(types: Vec<PrimitiveType>) -> Self {
        Self { types }
    }
}
