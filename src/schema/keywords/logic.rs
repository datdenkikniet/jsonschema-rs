use crate::{
    json::{Json, Key},
    schema::{Annotation, AnnotationValue, JsonSchema, JsonSchemaValidator},
};

#[derive(Debug, Clone, PartialEq)]
pub struct LogicError<'schema> {
    pub key: Key,
    pub schema: &'schema LogicApplier<'schema>,
    pub kind: LogicErrorKind,
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

#[derive(Debug, Clone, PartialEq)]
pub enum LogicErrorKind {
    AllOfMissing,
    AnyOfMissing,
    OneOfMissing,
    OneOfMoreThanOne,
    NotIs,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicApplier<'schema> {
    AllOf(Vec<JsonSchema<'schema>>),
    AnyOf(Vec<JsonSchema<'schema>>),
    OneOf(Vec<JsonSchema<'schema>>),
    Not(JsonSchema<'schema>),
}

impl<'me> JsonSchemaValidator for LogicApplier<'me> {
    fn validate_json<'schema>(
        &'schema self,
        key_to_input: Key,
        input: &Json,
        annotations: &mut Vec<Annotation<'schema>>,
    ) -> bool {
        let mut success = true;
        let schemas = match self {
            LogicApplier::AllOf(schemas)
            | LogicApplier::AnyOf(schemas)
            | LogicApplier::OneOf(schemas) => schemas,
            LogicApplier::Not(schema) => {
                if schema.validate_json(key_to_input.copy_of(), input, annotations) {
                    annotations.push(
                        LogicError {
                            schema: self,
                            key: key_to_input.copy_of(),
                            kind: LogicErrorKind::NotIs,
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
            if schema.validate_json(key_to_input.copy_of(), input, annotations) {
                valid += 1;
            }
        }

        match self {
            LogicApplier::AllOf(_) => {
                if valid != total_size {
                    annotations.push(
                        LogicError {
                            schema: self,
                            key: key_to_input.copy_of(),
                            kind: LogicErrorKind::AllOfMissing,
                        }
                        .into(),
                    );
                    success = false;
                }
            }
            LogicApplier::AnyOf(_) => {
                if valid == 0 {
                    annotations.push(
                        LogicError {
                            schema: self,
                            key: key_to_input.copy_of(),
                            kind: LogicErrorKind::AnyOfMissing,
                        }
                        .into(),
                    );
                    success = false;
                }
            }
            LogicApplier::OneOf(_) => {
                if valid == 0 {
                    annotations.push(
                        LogicError {
                            schema: self,
                            key: key_to_input.copy_of(),
                            kind: LogicErrorKind::OneOfMissing,
                        }
                        .into(),
                    );
                    success = false;
                } else if valid != 1 {
                    annotations.push(
                        LogicError {
                            schema: self,
                            key: key_to_input.copy_of(),
                            kind: LogicErrorKind::OneOfMoreThanOne,
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

#[cfg(test)]
mod tests {
    use super::LogicApplier;
    use crate::json::{Json, Key};
    use crate::schema::{JsonSchema, JsonSchemaValidator};

    macro_rules! assert_pretty_print {
        ($applier: expr, $test: expr, $input: expr) => {
            let errors = &mut Vec::new();
            let key = Key::default();
            assert!(
                $applier.validate_json(key, &$input, errors) == $test,
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
                let not_present: Json = "Not present".into();

                let me = JsonSchema::from_primitive(&input);
                let not_me = JsonSchema::from_primitive(&not_present);

                let applier = $applier(vec![me.clone()]);
                assert_pretty_print!(applier, $self_only, input.clone());

                let applier = $applier(vec![me.clone(), not_me.clone()]);
                assert_pretty_print!(applier, $self_and_other, input);

                let applier = $applier(vec![me.clone(), me.clone()]);
                assert_pretty_print!(applier, $self_twice, input);

                let applier = $applier(vec![not_me.clone()]);
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
        let not_present: Json = "Not present".into();

        let me = JsonSchema::from_primitive(&input);
        let not_me = JsonSchema::from_primitive(&not_present);

        let applier = LogicApplier::Not(me);
        assert_pretty_print!(applier, false, input);

        let applier = LogicApplier::Not(not_me);
        assert_pretty_print!(applier, true, input);
    }
}
