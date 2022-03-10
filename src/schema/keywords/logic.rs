use crate::{
    json::{Json, Key},
    schema::{Annotation, AnnotationValue, JsonSchema, JsonSchemaValidator},
};

#[derive(Debug, Clone)]
pub struct LogicError<'schema> {
    pub key: Key,
    pub schema: JsonSchema<'schema>,
    pub kind: LogicErrorKind<'schema>,
}

impl<'schema> Into<Annotation<'schema>> for LogicError<'schema> {
    fn into(self) -> Annotation<'schema> {
        Annotation::LogicError(self)
    }
}

impl<'schema> AnnotationValue for LogicError<'schema> {
    fn is_error(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub enum LogicErrorKind<'schema> {
    AllOfMissing(&'schema Vec<&'schema JsonSchema<'schema>>),
    AnyOfMissing(&'schema Vec<&'schema JsonSchema<'schema>>),
    OneOfMissing(&'schema Vec<&'schema JsonSchema<'schema>>),
    OneOfMoreThanOne(&'schema Vec<&'schema JsonSchema<'schema>>),
    NotIs(&'schema JsonSchema<'schema>),
}

#[derive(Debug, Clone)]
pub enum LogicApplier<'schema> {
    AllOf(Vec<&'schema JsonSchema<'schema>>),
    AnyOf(Vec<&'schema JsonSchema<'schema>>),
    OneOf(Vec<&'schema JsonSchema<'schema>>),
    Not(&'schema JsonSchema<'schema>),
}

impl<'schema> Into<JsonSchema<'schema>> for LogicApplier<'schema> {
    fn into(self) -> JsonSchema<'schema> {
        JsonSchema::Logic(self)
    }
}

impl<'me> JsonSchemaValidator for LogicApplier<'me> {
    fn validate_json<'schema>(
        &'schema self,
        input: &Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool {
        let mut success = true;
        let key = Key::default();
        let schemas = match self {
            LogicApplier::AllOf(schemas)
            | LogicApplier::AnyOf(schemas)
            | LogicApplier::OneOf(schemas) => schemas,
            LogicApplier::Not(schema) => {
                if schema.validate_json(input, annotations) {
                    annotations.push(
                        LogicError {
                            schema: self.clone().into(),
                            key: key.clone(),
                            kind: LogicErrorKind::NotIs(schema),
                        }
                        .into(),
                    );
                    success = false;
                };
                return success;
            }
        };

        let total_size = schemas.iter().count();

        let mut valid = 0;
        for schema in schemas {
            if schema.validate_json(input, annotations) {
                valid += 1;
            }
        }

        match self {
            LogicApplier::AllOf(vec) => {
                if valid != total_size {
                    annotations.push(
                        LogicError {
                            schema: self.clone().into(),
                            key: key.copy_of(),
                            kind: LogicErrorKind::AllOfMissing(vec),
                        }
                        .into(),
                    );
                    success = false;
                }
            }
            LogicApplier::AnyOf(vec) => {
                if valid == 0 {
                    annotations.push(
                        LogicError {
                            schema: self.clone().into(),
                            key: key.copy_of(),
                            kind: LogicErrorKind::AnyOfMissing(vec),
                        }
                        .into(),
                    );
                    success = false;
                }
            }
            LogicApplier::OneOf(vec) => {
                if valid == 0 {
                    annotations.push(
                        LogicError {
                            schema: self.clone().into(),
                            key: key.copy_of(),
                            kind: LogicErrorKind::OneOfMissing(vec),
                        }
                        .into(),
                    );
                    success = false;
                } else if valid != 1 {
                    annotations.push(
                        LogicError {
                            schema: self.clone().into(),
                            key: key.copy_of(),
                            kind: LogicErrorKind::OneOfMoreThanOne(vec),
                        }
                        .into(),
                    );
                    success = false;
                }
            }
            LogicApplier::Not(_) => unreachable!(),
        }
        success
    }
}

#[derive(Debug, Clone)]
pub enum LogicValidationError<'schema> {
    SchemaArrayEmpty(LogicApplier<'schema>),
}

impl<'schema> LogicApplier<'schema> {
    /// Check if this applier itself is valid
    pub fn is_valid(&self) -> Result<(), LogicValidationError> {
        match self {
            LogicApplier::AllOf(data) | LogicApplier::AnyOf(data) | LogicApplier::OneOf(data) => {
                if data.is_empty() {
                    return Err(LogicValidationError::SchemaArrayEmpty(self.clone()));
                }
            }
            LogicApplier::Not(_) => {}
        }
        Ok(())
    }
}

#[cfg(test)]
macro_rules! assert_pretty_print {
    ($applier: expr, $test: expr, $input: expr) => {
        let errors = &mut Vec::new();
        assert!(
            $applier.validate_json(&$input, errors) == $test,
            "Failed: {:?} = {:?} not {}",
            $input,
            $applier,
            stringify!($test)
        )
    };
}

macro_rules! test {
    ($name: ident, $applier: expr, $self_only: ident, $self_and_other: ident, $self_twice: ident, $only_other: ident) => {
        #[test]
        fn $name() {
            let input: Json = "Test".into();

            let me = &input.clone().into();
            let not_me = &"Not present".into();

            let applier = $applier(vec![me]);
            assert_pretty_print!(applier, $self_only, input);

            let applier = $applier(vec![me, not_me]);
            assert_pretty_print!(applier, $self_and_other, input);

            let applier = $applier(vec![me, me]);
            assert_pretty_print!(applier, $self_twice, input);

            let applier = $applier(vec![not_me]);
            assert_pretty_print!(applier, $only_other, input);
        }
    };
}

test!(all_of, LogicApplier::AllOf, true, false, true, false);
test!(any_of, LogicApplier::AnyOf, true, true, true, false);
test!(one_of, LogicApplier::OneOf, true, true, false, false);

#[test]
fn not() {
    let input: Json = "Test".into();

    let me = &input.clone().into();
    let not_me = &"Not present".into();

    let applier = LogicApplier::Not(me);
    assert_pretty_print!(applier, false, input);

    let applier = LogicApplier::Not(not_me);
    assert_pretty_print!(applier, true, input);
}
