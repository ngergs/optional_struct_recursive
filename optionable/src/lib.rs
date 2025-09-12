//! # optionable
//!
//! Library to derive structs/enums with all fields recursively replaced with `Option`-variants.
//!
//! One common problem when expressing patches e.g. for [Kubernetes apply configurations](https://pkg.go.dev/k8s.io/client-go/applyconfigurations).
//! is that one would need for a given rust struct `T` a corresponding struct `TOpt` where all fields are optional.
//! While trivial to write for plain structures this quickly becomes tedious for nested structs/enums.
//!
//! ## Deriving optional structs/enums
//!
//! The core utility of this library is to provide an [`derive@Optionable`]-derive macro that derives such an optioned type.
//! It supports nested structures as well as various container and pointer wrapper.
//! The general logic is the same as for other rust derives, If you want to use the [`derive@Optionable`]-derive macro for a struct/enum
//! every field of it needs to also have implemented the corresponding [`trait@Optionable`] trait (see below):
//! ```rust
//! # use optionable::Optionable;
//! # use serde::{Serialize,Deserialize};
//! #
//! #[derive(Optionable)]
//! #[optionable(derive(Serialize,Deserialize))]
//! struct DeriveExample {
//!     name: String,
//!     addresses: Vec<Address>,
//! }
//! #[derive(Optionable)]
//! #[optionable(derive(Serialize,Deserialize))]
//! struct Address {
//!     street_name: String,
//!     number: u8,
//! }
//! ```
//!
//! The generated optioned struct is (shown here with resolved associated types):
//!  ```rust
//! # use serde::{Serialize,Deserialize};
//! #[derive(Serialize,Deserialize)]
//! struct DeriveExampleOpt {
//!     name: Option<String>,
//!     addresses: Option<Vec<AddressOpt>>,
//! }
//! #[derive(Serialize,Deserialize)]
//! struct AddressOpt {
//!     street_name: Option<String>,
//!     number: Option<u8>,
//! }
//! ```
//!
//! ### Also works for enums
//! Enums are also supported for the derive macro, e.g.
//!
//! ```rust
//! # use optionable::Optionable;
//! #[derive(Optionable)]
//! enum DeriveExample {
//!     Unit,
//!     Plain(String),
//!     Address { street: String, number: u32 },
//!     Address2(String, u32),
//! }
//! ```
//! generates the following enum (shown here with resolved associated types):
//! ```rust
//! enum DeriveExampleOpt {
//!     Unit,
//!     Plain(Option<String>),
//!     Address { street: Option<String>, number: Option<u32> },
//!     AddressTuple(Option<String>, Option<u32>),
//! }
//! ```
//!
//! ## How it works
//! The main [`trait@Optionable`] trait is quite simple
//! ```rust
//! pub trait Optionable {
//!     type Optioned;
//! }
//! ```
//! It is a marker trait that allows to express for a given type `T` which type should be considered its `Optioned` type
//! such that `Option<Optioned>` would represent all variants of partial completeness.
//! For types without inner structure this means that the `Optioned` type will just resolve to the type itself, e.g.
//! ```rust,ignore
//! impl Optionable for String {
//!     type Optioned = String;
//! }
//! ```
//! For many primitive types as well as common wrapper or collection types the `Optionable`-trait is already implemented.

//! ## Crate features
//! - `chrono`: Derive [`trait@Optionable`] for types from [chrono](https://docs.rs/chrono/latest/chrono/).
//! - `serde_json`: Derive [`trait@Optionable`] for [serde_json](https://docs.rs/serde_json/latest/serde_json/)`::Value`.
//!
//! ## Limitations
//!
//! ### External types
//! Due to the orphan rule the usage of the library becomes cumbersome if one has a use case which heavily relies on crate-external types.
//! For well-established libraries adding corresponding `impl` to this crate (feature-gated) would be a worthwhile approach.
//!
//! ### IDE: Resolving associated types
//! Due to the use of associated types some IDE-hints do not fully resolve the associated types leaving you with
//! `<i32 as Optionable>::Optioned` instead of `i32`. Luckily, for checking type correctness and also for error messages
//! when using wrong types the associated types are resolved.
//!
//! ## Similar crates
//! Another crate with similar scope is [optional_struct](https://crates.io/crates/optional_struct).
//! It focuses specifically on structs (not enums) and offers a more manual approach, especially in respect to nested sub-struct,
//! providing many fine-grained configuration options.

#[doc(inline)]
pub use optionable_derive::Optionable;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
#[cfg(feature = "chrono")]
mod chrono;
#[cfg(feature = "serde_json")]
mod serde_json;

/// Marker trait that associated this type with a corresponding type where potential
/// inner sub-fields are recursively optional if possible for the given use case of the type.
/// Implementations of the trait can decide that some fields are also non-optional for the optioned type.
///
/// In detail this means that an `Option<T::Optioned>` should allow for every combination
/// of itself being set as well as just partial subfields of itself being set except
/// for fields that are always required.
/// Hence, for types without inner structure like `i32` the `Optioned` type will resolve to itself,
/// as e.g. `Option<i32>` already expresses the needed granularity.
pub trait Optionable {
    /// The associated type where fields (if possible for the given use case) are recursively optional.
    type Optioned;
}

// Blanket implementation for references to `Optionable` types.
impl<'a, T: Optionable> Optionable for &'a T {
    type Optioned = &'a T::Optioned;
}

/// Helper macro to generate an impl for `Optionalable` where the `Optioned` type
/// resolves to itself for types without inner structure like primitives (e.g. `i32`).
macro_rules! impl_optional_self {
    ($($t:ty),* $(,)?) => {
        $(impl Optionable for $t{
            type Optioned = Self;
        })*
    };
}
pub(crate) use impl_optional_self;

impl_optional_self!(
    // Rust primitives don't have inner structure, https://doc.rust-lang.org/rust-by-example/primitives.html
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, char, bool,
    // Other types without inner structure
    String, &str
);

/// Helper macro to generate an impl for `Optionable` for Containers.
/// Containers can be made optional by getting a corresponding container over the associated optional type.
macro_rules! impl_container {
    ($($t:ident),* $(,)?) => {
        $(impl<T: Optionable> Optionable for $t<T>{
            type Optioned = $t<T::Optioned>;
        })*
    };
}

impl_container!(
    Option,
    // Collections without an extra key, https://doc.rust-lang.org/std/collections/index.html
    Vec, VecDeque, LinkedList, BTreeSet, BinaryHeap, // Smart pointer and sync-container
    Box, Rc, Arc, RefCell, Mutex,
);

impl<T: Optionable, S> Optionable for HashSet<T, S> {
    type Optioned = HashSet<T::Optioned, S>;
}

impl<K, T: Optionable> Optionable for BTreeMap<K, T> {
    type Optioned = BTreeMap<K, T::Optioned>;
}

impl<K, T: Optionable, S> Optionable for HashMap<K, T, S> {
    type Optioned = HashMap<K, T::Optioned, S>;
}

#[cfg(test)]
mod tests {
    use crate::Optionable;
    use std::collections::{BTreeMap, HashMap};

    #[test]
    /// Check that an exemplary primitive type like `i32` resolves to itself as `Optioned` type.
    /// As all primitives share the same macro-generated code it does not add any value to iterate through
    /// all of them. If we missed a primitive type at the macro invocation we would also miss it at listing
    /// the types for the test.
    fn primitive_types_optioned_self() {
        let a: i32 = 10;
        let _: <i32 as Optionable>::Optioned = a;
    }

    #[test]
    /// Check that &str implements `Optionable`.
    fn str() {
        let a = "hello";
        let _: <&str as Optionable>::Optioned = a;
    }

    #[test]
    /// Check that pointer to `Optionable` types implement optionable.
    fn ptr() {
        let a = 2;
        let _: <&i32 as Optionable>::Optioned = &a;
    }

    #[test]
    /// Check that `Vec` implements optionable as an example container.
    fn container() {
        let a = vec![1, 2, 3];
        let _: <Vec<i64> as Optionable>::Optioned = a;
    }

    #[test]
    /// Check that `HashMap` and `BTreeMap` implements optionable.
    fn map() {
        let a = HashMap::from([(1, "a".to_owned())]);
        let _: <HashMap<i32, String> as Optionable>::Optioned = a;

        let a = BTreeMap::from([(1, "a".to_owned())]);
        let _: <BTreeMap<i32, String> as Optionable>::Optioned = a;
    }
}
