use crate::{
    json::{Json, Key},
    schema::{Annotation, JsonSchemaValidator},
};

#[derive(Debug, Clone)]
pub struct EnumError {
    pub key: Key,
}

impl<'schema> Into<Annotation<'schema>> for EnumError {
    fn into(self) -> Annotation<'schema> {
        Annotation::EnumError(self)
    }
}

#[derive(Debug, Clone)]
pub struct Enum {
    allowed_values: Vec<Json>,
}

impl JsonSchemaValidator for Enum {
    fn validate_json<'schema>(
        &'schema self,
        key_to_input: Key,
        input: &Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool {
        let success = self
            .allowed_values
            .iter()
            .find(|val| val == &input)
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

#[test]
fn test() {
    let enum_vals = Enum {
        allowed_values: vec!["a".into(), "b".into()],
    };

    let correct_value = "a".into();

    let key = Key::default();
    let annotations = &mut Vec::new();

    assert!(enum_vals.validate_json(key.copy_of(), &correct_value, annotations));

    let incorrect_value = "c".into();
    assert!(!enum_vals.validate_json(key, &incorrect_value, annotations));
}
