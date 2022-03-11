use crate::{
    json::{Json, Key, KeyPart},
    schema::{Annotation, AnnotationValue, JsonSchema, JsonSchemaValidator},
};

#[derive(Debug, Clone)]
pub enum PropertyErrorKind {
    IncorrectType,
    Missing,
    Invalid,
}

#[derive(Debug, Clone)]
pub struct PropertyError<'schema> {
    pub schema: JsonSchema<'schema>,
    pub key: Key,
    pub kind: PropertyErrorKind,
}

impl<'schema> AnnotationValue for PropertyError<'schema> {
    fn is_error(&self) -> bool {
        true
    }
}

impl<'schema> Into<Annotation<'schema>> for PropertyError<'schema> {
    fn into(self) -> Annotation<'schema> {
        Annotation::PropertyError(self)
    }
}

#[derive(Debug, Clone)]
pub struct Property<'schema> {
    required: bool,
    name: String,
    schemas: Vec<&'schema JsonSchema<'schema>>,
}

impl<'schema> Into<JsonSchema<'schema>> for Property<'schema> {
    fn into(self) -> JsonSchema<'schema> {
        JsonSchema::Properties(vec![self])
    }
}

impl<'me> JsonSchemaValidator for Property<'me> {
    fn validate_json<'schema>(
        &'schema self,
        key_to_input: &mut Key,
        input: &Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool {
        let object = match input {
            Json::Object(obj) => obj,
            _ => {
                annotations.push(
                    PropertyError {
                        schema: self.clone().into(),
                        key: key_to_input.copy_of(),
                        kind: PropertyErrorKind::IncorrectType,
                    }
                    .into(),
                );
                return false;
            }
        };

        if let Some((object_key, object_value)) = object.iter().find(|(key, _)| key == &&self.name)
        {
            let input_key = &mut key_to_input.copy_of();
            input_key.push(KeyPart::Identifier(object_key.clone()));

            let failures = self
                .schemas
                .iter()
                .filter(|schema| !schema.validate_json(input_key, object_value, annotations))
                .count();
            if failures != 0 {
                annotations.push(
                    PropertyError {
                        schema: self.clone().into(),
                        key: input_key.clone(),
                        kind: PropertyErrorKind::Invalid,
                    }
                    .into(),
                );
                false
            } else {
                true
            }
        } else {
            annotations.push(
                PropertyError {
                    schema: self.clone().into(),
                    key: key_to_input.copy_of(),
                    kind: PropertyErrorKind::Missing,
                }
                .into(),
            );
            !self.required
        }
    }
}

impl<'schema> Property<'schema> {
    pub fn new(name: &str, schemas: Vec<&'schema JsonSchema<'schema>>, required: bool) -> Self {
        Self {
            name: name.to_string(),
            schemas,
            required,
        }
    }

    pub fn set_required(&mut self, required: bool) {
        self.required = required;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        json::{Json, Key},
        schema::{keywords::Type, JsonSchema, JsonSchemaValidator},
    };

    use super::Property;

    #[test]
    fn required() {
        let input = &Json::from_string(r#"{"x": "value"}"#).unwrap();

        let ty = vec![&JsonSchema::Type(Type::String)];

        let mut schema = Property {
            required: false,
            name: "x".to_string(),
            schemas: ty,
        };

        macro_rules! test {
            ($name: expr, $required: expr, $success: expr, $empty: expr) => {
                let annotations = &mut Vec::new();
                schema.name = $name.to_string();
                schema.required = $required;
                let key = &mut Key::default();
                let result = schema.validate_json(key, &input, annotations);
                assert_eq!(result, $success);
                assert_eq!(annotations.is_empty(), $empty);
            };
        }

        test!("x", false, true, true);
        test!("x", true, true, true);
        test!("y", false, true, false);
        test!("y", true, false, false);
    }

    #[test]
    fn incorrect_type() {
        let input = &Json::from_string(r#"["x", "value"]"#).unwrap();

        let ty = vec![&JsonSchema::Type(Type::String)];

        let schema = Property {
            required: false,
            name: "x".to_string(),
            schemas: ty,
        };

        let annotations = &mut Vec::new();
        let key = &mut Key::default();
        let result = schema.validate_json(key, input, annotations);

        assert!(!result);
        assert!(!annotations.is_empty());
    }
}
