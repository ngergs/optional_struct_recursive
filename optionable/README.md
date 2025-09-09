# optionable

One common problem when expressing patches e.g. for [Kubernetes apply configurations](https://pkg.go.dev/k8s.io/client-go/applyconfigurations).
is that one would need for a given rust struct `T` a corresponding struct `TOpt` where all fields are optional.
While trivial to write for plain structures this quickly becomes tedious for nested structs/enums.

## Deriving optional structs/enums

The core utility of this library is to provide an `Optionable`-derive macro that derives such an optioned type.
It supports nested structures as well as various container and pointer wrapper.

The general logic is the same as for other rust derives, If you want to use the derive `Optionable` for a struct/enum
every field of it needs to also have implemented the corresponding `Optionable` trait (see below):
```rust
#[derive(Optionable)]
#[optionable(derive(Serialize,Deserialize))]
struct DeriveExample {
    name: String,
    addresses: Vec<Address>,
}
#[derive(Optionable)]
#[optionable(derive(Serialize,Deserialize))]
struct Address {
    street_name: String,
    number: u8,
}
```

The generated optioned struct is (shortened and simplified):
```rust
struct DeriveExampleOpt {
    name: Option<String>,
    addresses: Option<Vec<AddressOpt>>,
}
struct AddressOpt {
    street_name: Option<String>,
    number: Option<u8>,
}
```

## How it works
The main `Optionable` trait is quite simple
```rust
pub trait Optionable {
    type Optioned;
}
```
It is a marker trait that allows to express for a given type `T` which type should be considered its `Optioned` type
such that `Option<Optioned>` would represent all variants of partial completeness.
For types without inner structure this means that the `Optioned` type will just resolve to the type itself, e.g.
```rust
impl Optionable for String {
    type Optioned = String;
}
```
For many primitive types as well as common wrapper or collection types the `Optionable`-trait is already implemented.

## Crate features
- `chrono`: Derive `Optionable` for types from [chrono](https://docs.rs/chrono/latest/chrono/)
- `serde_json`: Derive `Optionable` for [serde_json](https://docs.rs/serde_json/latest/serde_json/)::Value

## Limitations

### External types
Due to the orphan rule the usage of the library becomes cumbersome if one has a use case which heavily relies on crate-external types.
For well-established libraries adding corresponding `impl` to this crate (feature-gated) would be a worthwhile approach.

### IDE: Resolving associated types
Due to the use of associated types some IDE-hints do not fully resolve the associated types leaving you with
`<i32 as Optionable>::Optioned` instead of `i32`. Luckily, for checking type correctness and also for error messages
when using wrong types the associated types are resolved.

## Similar crates
That I'm aware of:
- [optional_struct](https://crates.io/crates/optional_struct): Limit to structs, follows a more manual approach
  with many granular configuration options, does not support automatic detection of recursive optional sub-structs.