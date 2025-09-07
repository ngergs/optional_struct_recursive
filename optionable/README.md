# optionable

Tooling to derive structs/enums with all fields recurisvely replaced with `Option`-variants.

One common problem when expressing patches e.g. for [Kubernetes apply configurations](https://pkg.go.dev/k8s.io/client-go/applyconfigurations).
is that one would need for a given rust struct `T` a corresponding struct where all fields are optional.
While trivial to write for plain structures this quickly becomes tedious for nested structs/enums.

## Deriving optional structs/enums

The core utility of this library is to provide an `Optionable`-derive macro
that derives such an optioned type. It supports nested structures as well as various
container and pointer wrapper. The general logic is the same as for other rust derives:
If you want to derive `Optionable` for a type using a struct/enum as a subfield the
given struct/enum needs to also derive (or implement) `Optionable`:
```rust
#[derive(Optionable)]
struct DeriveExample {
    name: String,
    addresses: Vec<Address>,
}
#[derive(Optionable)]
struct Address {
    street_name: String,
    number: u8,
}
```
The generated code is (shortened and simplified):
```rust
struct DeriveExampleOpt {
    name: Option<String>,
    addresses: Option<Vec<AddressOpt>>,
}
struct AddressOpt {
    street_name: Option<String>,
    number: Option<u8>,
}
``````

## How it works
The main trait is quite simple
```rust
pub trait Optionable {
    type Optioned;
}
```
It is basically a marker trait that let's express for a given type `T` which type should be considered it's `Optioned` type
such that `Option<Optioned>` would represent all variants of partial completeness.
For types without inner structure this means that the `Optioned` type will just be the type itself, e.g.
```rust
impl Optionable for String{
    type Optioned = String;
}
```
For many primitive types as well as common wrapper or collection types the `Optionable`-trait is already implemented.

## Limitations

### External types
Due to the orphan rule the usage of the library becomes cumbersome if one has a use case which heavily relies on crate-external types.
For well-established libraries adding corresponding `impl` to this crate (feature-gated) would be a worthwhile approach.

### IDE: Resolving associated types
Due to the use of associated types some IDE-hints do not fully resolve the associated types leaving you with
`<i32 as Optionable>::Optioned` instead of `i32`. Luckily, for checking type correctness and also for error messages
when using wrong types the associated types are resolved.