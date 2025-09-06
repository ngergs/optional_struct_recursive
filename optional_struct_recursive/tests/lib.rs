use optional_struct_recursive_derive::Optionable;

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
/// Check that the derive macro works.
fn derive_unnamed_struct() {
    #[derive(Optionable)]
    #[allow(dead_code)]
    struct DeriveExample(String, i32);

    let _ = DeriveExampleOpt(None, None);
    let _ = DeriveExampleOpt(Some("a".to_owned()), Some(42));
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

#[test]
/// Check that the derive macro works with nested structs
fn derive_nested() {
    #[derive(Optionable)]
    #[allow(dead_code)]
    struct DeriveExample {
        name: String,
        address: Address,
    }
    #[derive(Optionable)]
    #[allow(dead_code)]
    struct Address {
        street_name: String,
        number: u8,
    }

    let _ = DeriveExampleOpt {
        name: None,
        address: None,
    };
    let _ = DeriveExampleOpt {
        name: Some("a".to_owned()),
        address: Some(AddressOpt {
            street_name: None,
            number: None,
        }),
    };
    let _ = DeriveExampleOpt {
        name: Some("a".to_owned()),
        address: Some(AddressOpt {
            street_name: Some("B".to_owned()),
            number: Some(2),
        }),
    };
}
