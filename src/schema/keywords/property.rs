use crate::{
    json::{Json, Key, KeyPart},
    schema::{Annotation, AnnotationValue, JsonSchema, JsonSchemaValidator},
};

#[derive(Debug, Clone)]
pub enum PropertyErrorKind {
    IncorrectType,
    Missing { required: bool },
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
        match self.kind {
            PropertyErrorKind::Missing { required } => required,
            _ => true,
        }
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
    schema: &'schema JsonSchema<'schema>,
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

            let success = self
                .schema
                .validate_json(input_key, object_value, annotations);
            if !success {
                annotations.push(
                    PropertyError {
                        schema: self.schema.clone(),
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
                    kind: PropertyErrorKind::Missing {
                        required: self.required,
                    },
                }
                .into(),
            );
            !self.required
        }
    }
}

impl<'schema> Property<'schema> {
    pub fn new(name: &str, schema: &'schema JsonSchema<'schema>, required: bool) -> Self {
        Self {
            name: name.to_string(),
            schema,
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

        let ty = JsonSchema::Type(Type::String);

        let mut schema = Property {
            required: false,
            name: "x".to_string(),
            schema: &ty,
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

        let ty = JsonSchema::Type(Type::String);

        let schema = Property {
            required: false,
            name: "x".to_string(),
            schema: &ty,
        };

        let annotations = &mut Vec::new();
        let key = &mut Key::default();
        let result = schema.validate_json(key, input, annotations);

        assert!(!result);
        assert!(!annotations.is_empty());
    }
}
