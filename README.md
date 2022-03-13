# Feature list
The following features from draft 2020-12 should be added:
- [x] JSON tokenization & parsing
- [ ] JSON schema parsing from JSON
    - [ ] `$id`
    - [x] `$vocabulary`
    - [ ] `$ref`
    - [x] `$defs`
    - [x] Parse schemas in general
- [ ] Basic JSON schema keywords/validation/annotations
    - [x] JSON primitive comparison
    - [x] `properties`
    - [ ] `patternProperties`, `additionalProperties`, `propertyNames`
    - [x] `allOf`, `oneOf`, `anyOf`, `not`
    - [ ] `if`, `then`, `else`, `dependentSchemas`
    - [x] `prefixItems`, `items`, `contains`
    - [ ] `unevaluatedItems`, `unevaluatedProperties`
    - [ ] `title`
    - [ ] `description`
- [ ] Additional JSON schema keywords
    - [x] `enum`
    - [x] `type`
    - [ ] `const`
    - [ ] `multipleOf`, `maximum`, `exclusiveMaximum`, `minimum`, `inclusiveMinimum`
    - [ ]  `maxLength`, `minLength`, `pattern`
    - [ ]  `maxItems`, `minItems`, `uniqueItems`, `maxContains`, `minContains`
    - [ ]  `maxProperties`, `minProperties`, `required`, `dependentRequired`