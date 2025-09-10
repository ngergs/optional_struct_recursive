use optionable::Optionable;
use serde::Deserialize;
use serde::Serialize;

#[test]
/// Check that the derive macro works.
fn derive_named_struct() {
    #[derive(Optionable)]
    #[allow(dead_code)]
    struct DeriveExample {
        name: String,
        surname: String,
    }

    let _ = DeriveExampleOpt {
        name: None,
        surname: None,
    };
    let _ = DeriveExampleOpt {
        name: Some("a".to_owned()),
        surname: Some("b".to_owned()),
    };
}

#[test]
/// Named struct with a required field
fn derive_named_struct_required() {
    #[derive(Optionable)]
    #[allow(dead_code)]
    struct DeriveExample {
        name: String,
        #[optionable(required)]
        surname: String,
    }

    let _ = DeriveExampleOpt {
        name: None,
        surname: "b".to_owned(),
    };
    let _ = DeriveExampleOpt {
        name: Some("a".to_owned()),
        surname: "b".to_owned(),
    };
}

#[test]
/// Tuple
fn derive_unnamed_struct() {
    #[derive(Optionable)]
    #[allow(dead_code)]
    struct DeriveExample(String, i32);

    let _ = DeriveExampleOpt(None, None);
    let _ = DeriveExampleOpt(Some("a".to_owned()), Some(42));
}

#[test]
/// Tuple with a required field
fn derive_unnamed_struct_required() {
    #[derive(Optionable)]
    #[allow(dead_code)]
    struct DeriveExample(String, #[optionable(required)] i32);

    let _ = DeriveExampleOpt(None, 42);
    let _ = DeriveExampleOpt(Some("a".to_owned()), 42);
}

#[test]
/// Check that the derive macro works.
fn derive_generic() {
    #[derive(Optionable)]
    #[allow(dead_code)]
    struct DeriveExample<T, T2> {
        name: T,
        surname: T2,
    }

    let _ = DeriveExampleOpt::<i32, String> {
        name: None,
        surname: None,
    };
    let _ = DeriveExampleOpt::<i32, String> {
        name: Some(2),
        surname: Some("b".to_owned()),
    };
}

type _String = <String as ::optionable::Optionable>::Optioned;

#[test]
/// Check that the derive macro works with nested structs
fn derive_nested() {
    #[derive(Optionable)]
    #[allow(dead_code)]
    struct DeriveExample {
        name: String,
        addresses: Vec<Address>,
    }
    #[derive(Optionable)]
    #[allow(dead_code)]
    struct Address {
        street_name: String,
        number: u8,
    }

    let _ = DeriveExampleOpt {
        name: None,
        addresses: None,
    };
    let _ = DeriveExampleOpt {
        name: Some("a".to_owned()),
        addresses: Some(vec![AddressOpt {
            street_name: None,
            number: None,
        }]),
    };
    let _ = DeriveExampleOpt {
        name: Some("a".to_owned()),
        addresses: Some(vec![AddressOpt {
            street_name: Some("B".to_owned()),
            number: Some(2),
        }]),
    };
    let _ = DeriveExampleOpt {
        name: None,
        addresses: None,
    };
}

#[test]
/// Check that the derive macro works for enums
fn derive_enum() {
    #[derive(Optionable)]
    #[allow(dead_code)]
    enum DeriveExample {
        Unit,
        Plain(String),
        Address { street: String, number: u32 },
        Address2(String, u32),
    }

    // unit does not get optioned as it has no inner structure
    let _ = DeriveExampleOpt::Unit;

    let _ = DeriveExampleOpt::Plain(None);
    let _ = DeriveExampleOpt::Address {
        street: None,
        number: None,
    };
    let _ = DeriveExampleOpt::Address2(None, None);

    let _ = DeriveExampleOpt::Plain(Some("a".to_owned()));
    let _ = DeriveExampleOpt::Address {
        street: Some("a".to_owned()),
        number: Some(42),
    };
    let _ = DeriveExampleOpt::Address2(Some("a".to_owned()), Some(42));
}

#[test]
/// Check that forwarding other derives via helper attributes works
fn derive_forward_other_derives() {
    #[derive(Optionable)]
    #[optionable(derive(Deserialize, Serialize))]
    #[allow(dead_code)]
    struct DeriveExample {
        name: String,
        surname: String,
    }

    let a = DeriveExampleOpt {
        name: Some("a".to_owned()),
        surname: None,
    };
    let a_json = serde_json::to_string(&a).unwrap();
    assert_eq!(a_json, "{\"name\":\"a\"}");
}
