use crate::{
    json::{Json, Key},
    schema::{Annotation, JsonSchemaValidator},
};

#[derive(Debug, Clone, PartialEq)]
pub struct EnumError {
    pub key: Key,
}

impl<'schema> Into<Annotation<'schema>> for EnumError {
    fn into(self) -> Annotation<'schema> {
        Annotation::EnumError(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum<'schema> {
    allowed_values: Vec<&'schema Json>,
}

impl<'me> JsonSchemaValidator for Enum<'me> {

    fn validate_json<'schema>(
        &'schema self,
        key_to_input: Key,
        input: &Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool {
        let success = self
            .allowed_values
            .iter()
            .find(|val| val == &&input)
            .is_some();
        if !success {
            annotations.push(
                EnumError {
                    key: key_to_input.copy_of(),
                }
                .into(),
            )
        }
        success
    }
}

impl<'schema> Enum<'schema> {
    pub fn new(values: Vec<&'schema Json>) -> Self {
        Self {
            allowed_values: values,
        }
    }
}

#[test]
fn test() {
    let a = "a".into();
    let b = "b".into();
    let enum_vals = Enum {
        allowed_values: vec![&a, &b],
    };

    let correct_value = "a".into();

    let key = Key::default();
    let annotations = &mut Vec::new();

    assert!(enum_vals.validate_json(key.copy_of(), &correct_value, annotations));

    let incorrect_value = "c".into();
    assert!(!enum_vals.validate_json(key, &incorrect_value, annotations));
}
