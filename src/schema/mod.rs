pub mod keywords;

pub mod parser;

pub mod uri;

use std::collections::HashMap;

use crate::json::{Json, Key};

use keywords::{
    annotations::{EnumError, LogicError, PropertyError, TypeError},
    LogicApplier, Property, Type,
};

use self::{
    keywords::{annotations::ArrayError, Contains, Enum, Items, PrefixItems},
    uri::Uri,
};

macro_rules! get_if_is {
    ($input: expr, $is: path, $err: expr) => {
        match $input {
            $is(val) => val,
            _ => {
                $err();
                return false;
            }
        }
    };
}

pub(crate) use get_if_is;

trait JsonSchemaValidator {
    fn validate_json<'schema>(
        &'schema self,
        key_to_input: Key,
        input: &Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool;
}

trait AnnotationValue {
    fn is_error(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Annotation<'schema> {
    LogicError(LogicError<'schema>),
    PropertyError(PropertyError<'schema>),
    TypeError(TypeError),
    EnumError(EnumError),
    ItemsError(ArrayError),
    Unequal {
        schema: &'schema JsonSchema<'schema>,
        key: Key,
    },
    PrefixItemsLen(Key, usize),
}

impl<'schema> From<ArrayError> for Annotation<'schema> {
    fn from(e: ArrayError) -> Self {
        Self::ItemsError(e)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsonSchema<'schema> {
    id: Option<Uri>,
    vocabulary: Option<HashMap<Uri, bool>>,
    defs: Option<HashMap<String, JsonSchema<'schema>>>,
    schemas: Vec<RootSchema<'schema>>,
    unknowns: HashMap<String, &'schema Json>,
}

impl<'schema> JsonSchema<'schema> {
    pub fn new(
        id: Option<Uri>,
        vocabulary: Option<HashMap<Uri, bool>>,
        defs: Option<HashMap<String, JsonSchema<'schema>>>,
        schemas: Vec<RootSchema<'schema>>,
        unknowns: HashMap<String, &'schema Json>,
    ) -> Self {
        Self {
            id,
            vocabulary,
            defs,
            schemas,
            unknowns,
        }
    }

    pub fn with_root_schemas(schemas: Vec<RootSchema<'schema>>) -> Self {
        Self {
            id: None,
            vocabulary: None,
            defs: None,
            unknowns: HashMap::new(),
            schemas,
        }
    }

    pub fn from_primitive(primitive: &'schema Json) -> Self {
        Self {
            id: None,
            vocabulary: None,
            defs: None,
            unknowns: HashMap::new(),
            schemas: vec![RootSchema::Primitive(primitive)],
        }
    }

    pub fn id(&self) -> &Option<Uri> {
        &self.id
    }

    pub fn vocabulary(&self) -> &Option<HashMap<Uri, bool>> {
        &self.vocabulary
    }

    pub fn defs(&self) -> &Option<HashMap<String, JsonSchema>> {
        &self.defs
    }

    pub fn schemas(&self) -> &Vec<RootSchema> {
        &self.schemas
    }

    pub fn unknowns(&self) -> &HashMap<String, &'schema Json> {
        &self.unknowns
    }
}

impl<'schema> JsonSchema<'schema> {
    fn validate_json<'input>(
        &'schema self,
        key_to_input: Key,
        input: &'input Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool {
        let mut success = true;
        for schema in self.schemas() {
            if !schema.validate_json(self, key_to_input.copy_of(), input, annotations) {
                success = false;
            }
        }
        success
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RootSchema<'schema> {
    Ref(&'schema RootSchema<'schema>),
    Primitive(&'schema Json),
    Logic(LogicApplier<'schema>),
    Properties(Vec<Property<'schema>>),
    Type(Type),
    Enum(Enum<'schema>),
    Items(Items<'schema>),
    PrefixItems(PrefixItems<'schema>),
    Contains(Contains<'schema>),
}

impl<'schema> RootSchema<'schema> {
    fn validate_json<'input>(
        &'schema self,
        parent: &'schema JsonSchema,
        key_to_input: Key,
        input: &'input Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool {
        let success = match self {
            RootSchema::Ref(schema) => {
                schema.validate_json(parent, key_to_input, input, annotations)
            }
            RootSchema::Primitive(primitive) => {
                if &input != primitive {
                    annotations.push(
                        Annotation::Unequal {
                            schema: parent,
                            key: key_to_input.copy_of(),
                        }
                        .into(),
                    );
                    false
                } else {
                    true
                }
            }
            RootSchema::Logic(logic) => logic.validate_json(key_to_input, input, annotations),
            RootSchema::Properties(properties) => {
                let mut success = true;
                for property in properties {
                    if !property.validate_json(key_to_input.copy_of(), input, annotations) {
                        success = false;
                    }
                }
                success
            }
            RootSchema::Type(ty) => ty.validate_json(key_to_input, input, annotations),
            RootSchema::Enum(en) => en.validate_json(key_to_input, input, annotations),
            RootSchema::PrefixItems(items) => items.validate_json(key_to_input, input, annotations),
            RootSchema::Items(items) => items.validate_json(key_to_input, input, annotations),
            RootSchema::Contains(contains) => {
                contains.validate_json(key_to_input, input, annotations)
            }
        };

        success
    }
}

impl<'schema> Into<JsonSchema<'schema>> for RootSchema<'schema> {
    fn into(self) -> JsonSchema<'schema> {
        JsonSchema::with_root_schemas(vec![self])
    }
}

impl<'schema> From<Enum<'schema>> for RootSchema<'schema> {
    fn from(en: Enum<'schema>) -> Self {
        Self::Enum(en)
    }
}

impl<'schema> From<Type> for RootSchema<'schema> {
    fn from(en: Type) -> Self {
        Self::Type(en)
    }
}

impl<'schema> From<LogicApplier<'schema>> for RootSchema<'schema> {
    fn from(logic: LogicApplier<'schema>) -> Self {
        Self::Logic(logic)
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
        json::{Key, Lexer, Parser},
        schema::{
            keywords::{PrimitiveType, Property},
            JsonSchema, RootSchema,
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

        let string_type =
            JsonSchema::with_root_schemas(vec![RootSchema::Type(PrimitiveType::String.into())]);
        let number_type =
            JsonSchema::with_root_schemas(vec![RootSchema::Type(PrimitiveType::Number.into())]);

        let second_level = JsonSchema::with_root_schemas(vec![RootSchema::Properties(vec![
            Property::new("first_nested_key", vec![&number_type], false),
            Property::new("second_nested_key", vec![&string_type], false),
        ])]);

        let first_level = RootSchema::Properties(vec![
            Property::new("first_key", vec![&string_type], false),
            Property::new("second_key", vec![&second_level], false),
        ]);

        let first_level = JsonSchema::with_root_schemas(vec![first_level]);

        let annotations = &mut Vec::new();
        let validation = first_level.validate_json(Key::default(), &input, annotations);

        assert!(annotations.is_empty(), "{:?}", validation);
    }
}
