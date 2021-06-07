use crate::{
    ir::schema::Error,
    json_schema,
    json_schema::BsonType,
    map, schema,
    schema::{Atomic::*, Document, Satisfaction::*, Schema::*},
    set,
};
use std::convert::TryFrom;

macro_rules! test_satisfies {
    ($func_name:ident, $expected:expr, $self:expr, $other:expr $(,)?) => {
        #[test]
        fn $func_name() {
            let res = $self.satisfies(&$other);
            assert_eq!($expected, res)
        }
    };
}

macro_rules! test_from_json_schema {
    ($func_name:ident, $schema_schema:expr, $json_schema:expr) => {
        #[test]
        fn $func_name() {
            let s = schema::Schema::try_from($json_schema);
            assert_eq!(s, $schema_schema);
        }
    };
}

test_from_json_schema!(
    convert_bson_single_to_atomic,
    Ok(Atomic(Integer)),
    json_schema::Schema {
        bson_type: Some(BsonType::Single("int".to_string())),
        ..Default::default()
    }
);

test_from_json_schema!(
    invalid_bson_type,
    Err(Error::InvalidJsonSchema(
        "blah is not a valid BSON type".to_string()
    )),
    json_schema::Schema {
        bson_type: Some(BsonType::Single("blah".to_string())),
        ..Default::default()
    }
);

test_from_json_schema!(
    convert_bson_multiple_to_any_of,
    Ok(AnyOf(vec![Atomic(Integer), Atomic(Null)])),
    json_schema::Schema {
        bson_type: Some(BsonType::Multiple(vec![
            "int".to_string(),
            "null".to_string()
        ])),
        ..Default::default()
    }
);

test_from_json_schema!(
    convert_one_of_to_one_of,
    Ok(OneOf(vec![Atomic(Integer), Atomic(Null)])),
    json_schema::Schema {
        one_of: Some(vec![
            json_schema::Schema {
                bson_type: Some(BsonType::Single("int".to_string())),
                ..Default::default()
            },
            json_schema::Schema {
                bson_type: Some(BsonType::Single("null".to_string())),
                ..Default::default()
            }
        ]),
        ..Default::default()
    }
);

test_from_json_schema!(
    one_of_invalid_nested_bson,
    Err(Error::InvalidJsonSchema(
        "blah is not a valid BSON type".to_string()
    )),
    json_schema::Schema {
        one_of: Some(vec![
            json_schema::Schema {
                bson_type: Some(BsonType::Single("blah".to_string())),
                ..Default::default()
            },
            json_schema::Schema {
                bson_type: Some(BsonType::Single("null".to_string())),
                ..Default::default()
            }
        ]),
        ..Default::default()
    }
);

test_from_json_schema!(
    one_of_invalid_extra_fields,
    Err(Error::InvalidJsonSchema(
        "invalid combination of fields".to_string()
    )),
    json_schema::Schema {
        bson_type: Some(BsonType::Single("int".to_string())),
        one_of: Some(vec![json_schema::Schema {
            bson_type: Some(BsonType::Single("null".to_string())),
            ..Default::default()
        }]),
        ..Default::default()
    }
);

test_from_json_schema!(
    convert_any_of_to_any_of,
    Ok(AnyOf(vec![Atomic(Integer), Atomic(Null)])),
    json_schema::Schema {
        any_of: Some(vec![
            json_schema::Schema {
                bson_type: Some(BsonType::Single("int".to_string())),
                ..Default::default()
            },
            json_schema::Schema {
                bson_type: Some(BsonType::Single("null".to_string())),
                ..Default::default()
            }
        ]),
        ..Default::default()
    }
);

test_from_json_schema!(
    convert_properties_to_document,
    Ok(Document(Document {
        keys: map![
            "a".to_string() => Atomic(Integer),
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: true,
    })),
    json_schema::Schema {
        bson_type: Some(BsonType::Single("object".to_string())),
        properties: Some(map! { "a".to_string() => json_schema::Schema {
            bson_type: Some(BsonType::Single("int".to_string())),
            ..Default::default()
        }, "b".to_string() => json_schema::Schema {
            bson_type: Some(BsonType::Single("int".to_string())),
            ..Default::default()
        }}),
        required: Some(vec!["a".to_string()]),
        additional_properties: Some(true),
        ..Default::default()
    }
);

test_from_json_schema!(
    document_bson_type_not_object,
    Ok(Atomic(Integer)),
    json_schema::Schema {
        bson_type: Some(BsonType::Single("int".to_string())),
        properties: Some(map! { "a".to_string() => json_schema::Schema {
            bson_type: Some(BsonType::Single("int".to_string())),
            ..Default::default()
        }, "b".to_string() => json_schema::Schema {
            bson_type: Some(BsonType::Single("int".to_string())),
            ..Default::default()
        }}),
        required: Some(vec!["a".to_string()]),
        additional_properties: Some(true),
        ..Default::default()
    }
);

test_from_json_schema!(
    document_properties_not_set,
    Ok(Document(Document {
        keys: map![],
        required: set!["a".to_string()],
        additional_properties: true
    })),
    json_schema::Schema {
        bson_type: Some(BsonType::Single("object".to_string())),
        required: Some(vec!["a".to_string()]),
        additional_properties: Some(true),
        ..Default::default()
    }
);

test_from_json_schema!(
    document_additional_properties_not_set,
    Ok(Document(Document {
        keys: map![
            "a".to_string() => Atomic(Integer),
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: true,
    })),
    json_schema::Schema {
        bson_type: Some(BsonType::Single("object".to_string())),
        properties: Some(map! { "a".to_string() => json_schema::Schema {
            bson_type: Some(BsonType::Single("int".to_string())),
            ..Default::default()
        }, "b".to_string() => json_schema::Schema {
            bson_type: Some(BsonType::Single("int".to_string())),
            ..Default::default()
        }}),
        required: Some(vec!["a".to_string()]),
        ..Default::default()
    }
);

test_from_json_schema!(
    convert_array_to_any_of,
    Ok(Array(Box::new(Atomic(Integer)))),
    json_schema::Schema {
        bson_type: Some(BsonType::Single("array".to_string())),
        items: Some(Box::new(json_schema::Schema {
            bson_type: Some(BsonType::Single("int".to_string())),
            ..Default::default()
        })),
        ..Default::default()
    }
);

test_from_json_schema!(
    items_set_bson_type_not_array,
    Ok(AnyOf(vec![Atomic(Integer)])),
    json_schema::Schema {
        bson_type: Some(BsonType::Multiple(vec!["int".to_string(),])),
        items: Some(Box::new(json_schema::Schema {
            bson_type: Some(BsonType::Single("int".to_string())),
            ..Default::default()
        })),
        ..Default::default()
    }
);

test_from_json_schema!(
    bson_type_array_set_missing_items_field,
    Ok(Array(Box::new(Any))),
    json_schema::Schema {
        bson_type: Some(BsonType::Single("array".to_string())),
        ..Default::default()
    }
);

test_from_json_schema!(
    convert_array_and_document_fields,
    Ok(AnyOf(vec![
        Array(Box::new(Atomic(Integer))),
        Document(Document {
            keys: map![
                "a".to_string() => Atomic(Integer),
                "b".to_string() => Atomic(Integer),
            ],
            required: set!["a".to_string()],
            additional_properties: true,
        })
    ])),
    json_schema::Schema {
        bson_type: Some(BsonType::Multiple(vec![
            "array".to_string(),
            "object".to_string()
        ])),
        properties: Some(map! { "a".to_string() => json_schema::Schema {
            bson_type: Some(BsonType::Single("int".to_string())),
            ..Default::default()
        }, "b".to_string() => json_schema::Schema {
            bson_type: Some(BsonType::Single("int".to_string())),
            ..Default::default()
        }}),
        required: Some(vec!["a".to_string()]),
        additional_properties: Some(true),
        items: Some(Box::new(json_schema::Schema {
            bson_type: Some(BsonType::Single("int".to_string())),
            ..Default::default()
        })),
        ..Default::default()
    }
);

test_from_json_schema!(
    bson_type_object_set_missing_document_fields,
    Ok(AnyOf(vec![
        Array(Box::new(Atomic(Integer))),
        Document(Document {
            keys: map![],
            required: set![],
            additional_properties: true
        })
    ])),
    json_schema::Schema {
        bson_type: Some(BsonType::Multiple(vec![
            "array".to_string(),
            "object".to_string()
        ])),
        items: Some(Box::new(json_schema::Schema {
            bson_type: Some(BsonType::Single("int".to_string())),
            ..Default::default()
        })),
        ..Default::default()
    }
);

test_satisfies!(satisfies_any_must_satisfy_any, Must, Any, Any);
test_satisfies!(satisfies_missing_must_satisfy_any, Must, Missing, Any);
test_satisfies!(
    satisfies_any_of_empty_must_satisfy_atomic,
    Must,
    AnyOf(vec![]),
    Atomic(Integer)
);
test_satisfies!(
    satisfies_any_of_empty_must_satisfy_any_of_empty,
    Must,
    AnyOf(vec![]),
    AnyOf(vec![]),
);
test_satisfies!(
    satisfies_any_of_empty_must_satisfy_one_of_empty,
    Must,
    AnyOf(vec![]),
    OneOf(vec![]),
);
test_satisfies!(
    satisfies_one_of_empty_must_satisfy_atomic,
    Must,
    OneOf(vec![]),
    Atomic(Integer)
);
test_satisfies!(
    satisfies_any_of_empty_must_satisfy_missing,
    Must,
    AnyOf(vec![]),
    Missing,
);
test_satisfies!(
    satisfies_one_of_empty_must_satisfy_missing,
    Must,
    OneOf(vec![]),
    Missing,
);
test_satisfies!(
    satisfies_one_of_empty_must_satisfy_any_of_empty,
    Must,
    OneOf(vec![]),
    AnyOf(vec![]),
);
test_satisfies!(
    satisfies_one_of_empty_must_satisfy_one_of_empty,
    Must,
    OneOf(vec![]),
    OneOf(vec![]),
);
test_satisfies!(
    satisfies_missing_must_satisfy_missing,
    Must,
    Missing,
    Missing
);
test_satisfies!(
    satisfies_missing_must_satisfy_any_of,
    Must,
    Missing,
    AnyOf(vec![Missing])
);
test_satisfies!(
    satisfies_one_of_missing_may_satisfy_missing,
    May,
    OneOf(vec![Atomic(String), Missing, Atomic(Integer)]),
    Missing
);
test_satisfies!(
    satisfies_any_of_missing_may_satisfy_missing,
    May,
    AnyOf(vec![Atomic(Integer), Missing, Atomic(String)]),
    Missing
);
test_satisfies!(
    satisfies_missing_must_not_satisfy_atomic,
    Not,
    Missing,
    Atomic(String)
);
test_satisfies!(
    satisfies_missing_must_not_satisfy_array,
    Not,
    Missing,
    Array(Box::new(Any)),
);
test_satisfies!(
    satisfies_missing_must_not_satisfy_document,
    Not,
    Missing,
    Document(Document {
        keys: map![],
        required: set![],
        additional_properties: true,
    })
);
test_satisfies!(
    satisfies_missing_must_not_satisfy_any_of,
    Not,
    Missing,
    AnyOf(vec![Atomic(String), Atomic(Integer)])
);
test_satisfies!(
    satisfies_missing_must_not_satisfy_one_of,
    Not,
    Missing,
    OneOf(vec![Atomic(String), Atomic(Integer)])
);
test_satisfies!(satisfies_atomic_must_satisfy_any, Must, Atomic(String), Any);
test_satisfies!(satisfies_any_may_satisfy_atomic, May, Any, Atomic(String));
test_satisfies!(
    satisfies_array_of_any_does_not_satisfy_atomic,
    Not,
    Array(Box::new(Any)),
    Atomic(Integer),
);
test_satisfies!(
    satisfies_missing_does_not_satisfy_atomic,
    Not,
    Missing,
    Atomic(String),
);
test_satisfies!(
    satisfies_any_of_must_satisfy_any,
    Must,
    AnyOf(vec![Atomic(String), Atomic(Integer)]),
    Any,
);

test_satisfies!(
    satisfies_one_of_must_satisfy_one_of_when_equal,
    Must,
    OneOf(vec![Atomic(String), Atomic(Integer)]),
    OneOf(vec![Atomic(String), Atomic(Integer)]),
);
test_satisfies!(
    satisfies_one_of_may_satisfy_one_of_when_unequal,
    May,
    OneOf(vec![Atomic(Double), Atomic(Integer)]),
    OneOf(vec![Atomic(String), Atomic(Integer)]),
);
test_satisfies!(
    satisfies_one_of_may_satisfy_one_of_when_self_has_missing,
    May,
    OneOf(vec![Atomic(String), Missing]),
    OneOf(vec![Atomic(String), Atomic(Integer)]),
);
test_satisfies!(
    satisfies_one_of_must_satisfy_one_of_when_other_has_missing,
    Must,
    OneOf(vec![Atomic(String), Atomic(Integer)]),
    OneOf(vec![Atomic(String), Atomic(Integer), Missing]),
);
test_satisfies!(
    satisfies_one_of_must_not_satisfy_when_one_of_contains_any,
    Not,
    OneOf(vec![Atomic(String), Atomic(Integer)]),
    OneOf(vec![Atomic(String), Atomic(Integer), Any]),
);
test_satisfies!(
    satisfies_any_of_must_satisfy_when_any_of_contains_any,
    Must,
    OneOf(vec![Atomic(String), Atomic(Integer)]),
    AnyOf(vec![Atomic(String), Atomic(Integer), Any]),
);
test_satisfies!(
    satisfies_one_of_must_may_satisfy_one_of_when_self_has_any,
    May,
    OneOf(vec![Atomic(String), Any]),
    OneOf(vec![Atomic(String), Atomic(Integer)]),
);
test_satisfies!(
    satisfies_one_of_must_satisfy_subset_one_of,
    Must,
    OneOf(vec![Atomic(String), Atomic(Integer)]),
    OneOf(vec![Atomic(String), Atomic(Integer), Atomic(Double)]),
);
test_satisfies!(
    satisfies_one_of_may_satisfy_proper_superset_one_of,
    May,
    OneOf(vec![Atomic(String), Atomic(Integer), Atomic(Double)]),
    OneOf(vec![Atomic(String), Atomic(Integer)]),
);
test_satisfies!(
    satisfies_array_of_string_must_satisfy_one_of_array_of_int_or_array_of_string,
    Must,
    Array(Box::new(Atomic(String))),
    OneOf(vec![
        Array(Box::new(Atomic(String))),
        Array(Box::new(Atomic(Integer)))
    ]),
);
test_satisfies!(
    satisfies_array_of_string_or_int_may_satisfy_one_of_array_of_int_or_array_of_string,
    May,
    Array(Box::new(OneOf(vec![Atomic(String), Atomic(Integer),]))),
    OneOf(vec![
        Array(Box::new(Atomic(String))),
        Array(Box::new(Atomic(Integer)))
    ]),
);
test_satisfies!(
    satisfies_array_of_string_or_int_must_satisfy_array_of_string_or_int,
    Must,
    Array(Box::new(OneOf(vec![Atomic(String), Atomic(Integer),]))),
    Array(Box::new(OneOf(vec![Atomic(String), Atomic(Integer),]))),
);
test_satisfies!(
    satisfies_document_must_satify_same_document,
    Must,
    Document(Document {
        keys: map![
            "a".to_string() => Any,
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: true
    }),
    Document(Document {
        keys: map![
            "a".to_string() => Any,
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: true,
    }),
);
test_satisfies!(
    satisfies_document_may_satify_with_more_permissive_key_schema,
    May,
    Document(Document {
        keys: map![
            "a".to_string() => Any,
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: false,
    }),
    Document(Document {
        keys: map![
            "a".to_string() => Atomic(String),
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: false,
    }),
);
test_satisfies!(
    satisfies_document_must_not_satify_with_incompatable_key_schema,
    Not,
    Document(Document {
        keys: map![
            "a".to_string() => Atomic(Integer),
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: false,
    }),
    Document(Document {
        keys: map![
            "a".to_string() => Atomic(String),
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: false,
    }),
);
test_satisfies!(
    satisfies_document_may_satify_with_fewer_required_keys,
    May,
    Document(Document {
        keys: map![
            "a".to_string() => Atomic(Integer),
            "b".to_string() => Atomic(Integer),
        ],
        required: set![],
        additional_properties: false,
    }),
    Document(Document {
        keys: map![
            "a".to_string() => Atomic(Integer),
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: false,
    }),
);
test_satisfies!(
    satisfies_document_must_not_satify_with_missing_required_key,
    Not,
    Document(Document {
        keys: map![
            "b".to_string() => Atomic(Integer),
        ],
        required: set![],
        additional_properties: false,
    }),
    Document(Document {
        keys: map![
            "a".to_string() => Atomic(Integer),
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: false,
    }),
);
test_satisfies!(
    satisfies_document_may_satify_with_missing_required_key,
    May,
    Document(Document {
        keys: map![
            "b".to_string() => Atomic(Integer),
        ],
        required: set![],
        additional_properties: true,
    }),
    Document(Document {
        keys: map![
            "a".to_string() => Atomic(Integer),
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: true,
    }),
);
test_satisfies!(
    satisfies_document_must_satify_with_more_required_keys,
    Must,
    Document(Document {
        keys: map![
            "a".to_string() => Atomic(Integer),
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: false,
    }),
    Document(Document {
        keys: map![
            "a".to_string() => Atomic(Integer),
            "b".to_string() => Atomic(Integer),
        ],
        required: set![],
        additional_properties: false,
    }),
);
test_satisfies!(
    satisfies_document_may_satify_due_to_possible_extra_keys,
    May,
    Document(Document {
        keys: map![
            "a".to_string() => Atomic(Integer),
            "b".to_string() => Atomic(Integer),
        ],
        required: set![],
        additional_properties: true,
    }),
    Document(Document {
        keys: map![
            "a".to_string() => Atomic(Integer),
            "b".to_string() => Atomic(Integer),
        ],
        required: set![],
        additional_properties: false,
    }),
);
test_satisfies!(
    satisfies_document_satifies_multiple_one_of_results_in_not_satisfied,
    Not,
    Document(Document {
        keys: map![
            "a".to_string() => Any,
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: false,
    }),
    OneOf(vec![
        Document(Document {
            keys: map![
                "a".to_string() => Any,
                "b".to_string() => Atomic(Integer),
            ],
            required: set!["a".to_string()],
            additional_properties: false,
        }),
        Document(Document {
            keys: map![
                "b".to_string() => Atomic(Integer),
            ],
            required: set![],
            additional_properties: true,
        }),
    ]),
);
test_satisfies!(
    satisfies_document_satifies_multiple_any_of_results_in_must_satisfy,
    Must,
    Document(Document {
        keys: map![
            "a".to_string() => Any,
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: false,
    }),
    AnyOf(vec![
        Document(Document {
            keys: map![
                "a".to_string() => Any,
                "b".to_string() => Atomic(Integer),
            ],
            required: set!["a".to_string()],
            additional_properties: false,
        }),
        Document(Document {
            keys: map![
                "b".to_string() => Atomic(Integer),
            ],
            required: set![],
            additional_properties: false,
        }),
    ]),
);
test_satisfies!(
    satisfies_document_satifies_one_of_one_of_results_must_satisfy,
    Must,
    Document(Document {
        keys: map![
            "a".to_string() => Any,
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: false,
    }),
    OneOf(vec![
        Document(Document {
            keys: map![
                "a".to_string() => Any,
                "b".to_string() => Atomic(Integer),
            ],
            required: set!["a".to_string()],
            additional_properties: false,
        }),
        Document(Document {
            keys: map![
                "e".to_string() => Atomic(Integer),
            ],
            required: set![],
            additional_properties: false,
        }),
    ]),
);
test_satisfies!(
    satisfies_document_may_satisfy_when_key_schema_may_satisfy,
    May,
    Document(Document {
        keys: map![
            "a".to_string() => Any,
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: false,
    }),
    Document(Document {
        keys: map![
            "a".to_string() => Atomic(Integer),
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: false,
    }),
);
test_satisfies!(
    satisfies_array_may_satisfy_when_array_item_schema_may_satisfy,
    May,
    Array(Box::new(Any)),
    Array(Box::new(Atomic(Integer))),
);
test_satisfies!(
    satisfies_array_may_satisfy_when_array_item_schema_may_satisfy_multiple_one_of_array,
    May,
    Array(Box::new(Any)),
    OneOf(vec![
        Array(Box::new(Atomic(Integer))),
        Array(Box::new(Atomic(String))),
    ]),
);
test_satisfies!(
    satisfies_array_may_satisfy_when_array_item_schema_may_satisfy_multiple_array_one_of,
    May,
    Array(Box::new(Any)),
    Array(Box::new(OneOf(vec![Atomic(Integer), Atomic(Double),]),)),
);
test_satisfies!(
    satisfies_array_of_missing_does_not_satisfy_array_of_atomic,
    Not,
    Array(Box::new(Missing)),
    Array(Box::new(Atomic(Integer))),
);

macro_rules! test_contains_field {
    ($func_name:ident, $expected:expr, $self:expr, $other:expr $(,)?) => {
        #[test]
        fn $func_name() {
            let res = $self.contains_field($other);
            assert_eq!($expected, res)
        }
    };
}

test_contains_field!(contains_field_any_may_contain_field, May, Any, "a");
test_contains_field!(
    contains_field_missing_does_not_contain_field,
    Not,
    Missing,
    "a"
);
test_contains_field!(
    contains_field_document_must_contain_field,
    Must,
    Document(Document {
        keys: map![
            "a".to_string() => Any,
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: false,
    }),
    "a",
);
test_contains_field!(
    contains_field_document_may_contain_field,
    May,
    Document(Document {
        keys: map![
            "a".to_string() => Any,
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: false,
    }),
    "b",
);
test_contains_field!(
    contains_field_document_may_contain_field_due_to_additional_properties,
    May,
    Document(Document {
        keys: map![
            "a".to_string() => Any,
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: true,
    }),
    "foo",
);
test_contains_field!(
    contains_field_document_must_not_contain_field,
    Not,
    Document(Document {
        keys: map![
            "a".to_string() => Any,
            "b".to_string() => Atomic(Integer),
        ],
        required: set!["a".to_string()],
        additional_properties: false,
    }),
    "foo",
);
test_contains_field!(
    contains_field_atomic_must_not_contain_field,
    Not,
    Atomic(String),
    "foo",
);
test_contains_field!(
    contains_field_one_of_document_and_atomic_may_not_contain_field,
    Not,
    OneOf(vec![
        Document(Document {
            keys: map![
                "a".to_string() => Any,
                "b".to_string() => Atomic(Integer),
            ],
            required: set!["a".to_string()],
            additional_properties: false,
        }),
        Atomic(String),
    ]),
    "c",
);
test_contains_field!(
    contains_field_one_of_document_and_atomic_may_contain_field,
    May,
    OneOf(vec![
        Document(Document {
            keys: map![
                "a".to_string() => Any,
                "b".to_string() => Atomic(Integer),
            ],
            required: set!["a".to_string()],
            additional_properties: false,
        }),
        Atomic(String),
    ]),
    "b",
);
test_contains_field!(
    contains_field_one_of_document_and_document_must_contain_field,
    Must,
    OneOf(vec![
        Document(Document {
            keys: map![
                "a".to_string() => Any,
                "b".to_string() => Atomic(Integer),
            ],
            required: set!["b".to_string()],
            additional_properties: false,
        }),
        Document(Document {
            keys: map![
                "a".to_string() => Any,
                "b".to_string() => Atomic(String),
            ],
            required: set!["b".to_string()],
            additional_properties: false,
        }),
    ]),
    "b",
);
