use optional_struct_recursive_derive::Optionable;

#[test]
/// Check that the derive macro works.
fn derive() {
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
