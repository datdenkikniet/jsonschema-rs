use crate::{
    json::{Json, Key},
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
pub struct Properties<'schema> {
    properties: Vec<&'schema Property<'schema>>,
}

impl<'me> JsonSchemaValidator for Properties<'me> {
    fn validate_json<'schema>(
        &'schema self,
        input: &Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool {
        let mut success = false;
        for property in &self.properties {
            if !property.validate_json(input, annotations) {
                success = false;
            }
        }
        success
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
        JsonSchema::Property(self)
    }
}

impl<'me> JsonSchemaValidator for Property<'me> {
    fn validate_json<'schema>(
        &'schema self,
        input: &Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool {
        let json_key = Key::default();
        let object = match input {
            Json::Object(obj) => obj,
            _ => {
                annotations.push(
                    PropertyError {
                        schema: self.clone().into(),
                        key: json_key.clone(),
                        kind: PropertyErrorKind::IncorrectType,
                    }
                    .into(),
                );
                return false;
            }
        };

        if let Some((_object_key, object_value)) = object.iter().find(|(key, _)| key == &&self.name)
        {
            let success = self.schema.validate_json(object_value, annotations);
            if !success {
                annotations.push(
                    PropertyError {
                        schema: self.clone().into(),
                        key: json_key.clone(),
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
                    key: json_key.clone(),
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

#[cfg(test)]
mod tests {
    use crate::{
        json::Json,
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
                let result = schema.validate_json(&input, annotations);
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

        let result = schema.validate_json(input, annotations);
        assert!(!result);
        assert!(!annotations.is_empty());
    }
}
