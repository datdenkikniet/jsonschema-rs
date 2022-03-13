use crate::{
    json::{Json, Key},
    schema::{get_if_is, Annotation, JsonSchema, JsonSchemaValidator},
};

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayError {
    pub key: Key,
    pub kind: ArrayErrorKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayErrorKind {
    NotArray,
    PrefixItemMissing,
    DoesNotContain,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefixItems<'schema> {
    schemas: Vec<JsonSchema<'schema>>,
}

impl<'me> JsonSchemaValidator for PrefixItems<'me> {
    fn validate_json<'schema>(
        &'schema self,
        key_to_input: Key,
        input: &Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool {
        let mut success = true;
        let array = match input {
            Json::Array(array) => array,
            _ => {
                annotations.push(
                    ArrayError {
                        key: key_to_input.copy_of(),
                        kind: ArrayErrorKind::NotArray,
                    }
                    .into(),
                );
                return false;
            }
        };

        let mut values = array.iter();

        for i in 0..self.schemas.len() {
            let schema = &self.schemas[i];

            if let Some(value) = values.next() {
                if !schema.validate_json(key_to_input.copy_of().push_idx(i), value, annotations) {
                    success = false;
                }
            } else {
                success = false;
                annotations.push(
                    ArrayError {
                        key: key_to_input.copy_of().push_idx(i),
                        kind: ArrayErrorKind::PrefixItemMissing,
                    }
                    .into(),
                )
            }
        }

        annotations.push(Annotation::PrefixItemsLen(key_to_input, self.schemas.len()));

        success
    }
}

impl<'schema> PrefixItems<'schema> {
    pub fn new(schemas: Vec<JsonSchema<'schema>>) -> Self {
        Self { schemas }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Items<'schema> {
    schema: JsonSchema<'schema>,
}

impl<'me> JsonSchemaValidator for Items<'me> {
    fn validate_json<'schema>(
        &'schema self,
        key_to_input: Key,
        input: &Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool {
        let mut success = true;

        let items = get_if_is!(input, Json::Array, || annotations.push(
            ArrayError {
                key: key_to_input.copy_of(),
                kind: ArrayErrorKind::NotArray,
            }
            .into(),
        ));

        let start = if let Some(prefix_len) = annotations.iter().find_map(|annotation| {
            if let Annotation::PrefixItemsLen(key, len) = annotation {
                if key == &key_to_input {
                    Some(*len)
                } else {
                    None
                }
            } else {
                None
            }
        }) {
            prefix_len
        } else {
            0
        };

        for i in start..items.len() {
            let item = &items[i];
            if !self
                .schema
                .validate_json(key_to_input.copy_of().push_idx(i), item, annotations)
            {
                success = false;
            }
        }

        success
    }
}

impl<'schema> Items<'schema> {
    pub fn new(schema: JsonSchema<'schema>) -> Self {
        Self { schema }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Contains<'schema> {
    schema: JsonSchema<'schema>,
}

impl<'me> JsonSchemaValidator for Contains<'me> {
    fn validate_json<'schema>(
        &'schema self,
        key_to_input: Key,
        input: &Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool {
        let values = get_if_is!(input, Json::Array, || annotations.push(
            ArrayError {
                key: key_to_input,
                kind: ArrayErrorKind::NotArray,
            }
            .into()
        ));

        let mut contains = false;
        for i in 0..values.len() {
            let value = &values[i];
            if self
                .schema
                .validate_json(key_to_input.copy_of().push_idx(i), value, annotations)
            {
                contains = true;
            }
        }

        contains
    }
}

impl<'schema> Contains<'schema> {
    pub fn new(schema: JsonSchema<'schema>) -> Self {
        Self { schema }
    }
}

#[test]
fn prefix_items() {
    let items = &Json::from_string(r#"["hello", "there", "general"]"#).unwrap();

    let hello = "hello".into();
    let there = "there".into();
    let general = "general".into();

    let prefix_items = PrefixItems {
        schemas: vec![
            JsonSchema::from_primitive(&hello),
            JsonSchema::from_primitive(&there),
            JsonSchema::from_primitive(&general),
        ],
    };

    let annotations = &mut Vec::new();
    let key = Key::default();

    let result = prefix_items.validate_json(key, items, annotations);
    assert!(result);
    assert_eq!(
        *annotations,
        vec![Annotation::PrefixItemsLen(Key::default(), 3)]
    );
}

#[test]
fn items() {
    let input = &Json::from_string(r#"["hello"]"#).unwrap();

    let hello_text = "hello".into();

    let items = Items {
        schema: JsonSchema::from_primitive(&hello_text),
    };

    let annotations = &mut Vec::new();
    let key = Key::default();

    let result = items.validate_json(key, input, annotations);

    assert!(result);
    assert!(annotations.is_empty());
}
