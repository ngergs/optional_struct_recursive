//! # optionable
//!
//! (Documentation for the [optionable](todo) crate where this derive macro is used.
//! Tooling to derive structs/enums with all fields recursively replaced with `Option`-variants.
//!
//! One common problem when expressing patches e.g. for [Kubernetes apply configurations](https://pkg.go.dev/k8s.io/client-go/applyconfigurations).
//! is that one would need for a given rust struct `T` a corresponding struct `TOpt` where all fields are optional.
//! While trivial to write for plain structures this quickly becomes tedious for nested structs/enums.
//!
//! ## Deriving optional structs/enums
//!
//! The core utility of this library is to provide an [`derive@Optionable`]-derive macro that derives such an optioned type.
//! It supports nested structures as well as various container and pointer wrapper.
//! The general logic is the same as for other rust derives, If you want to use the derive [`derive@Optionable`] for a struct/enum
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
//! The generated optioned struct is (shortened and simplified):
//!  ```rust
//!  struct DeriveExampleOpt {
//!     name: Option<String>,
//!     addresses: Option<Vec<AddressOpt>>,
//! }
//! struct AddressOpt {
//!     street_name: Option<String>,
//!     number: Option<u8>,
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
//! - `serde_json`: Derive [`trait@Optionable`] for [serde_json](https://docs.rs/serde_json/latest/serde_json/)::Value.
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

/// Marker trait that signals that this struct has a corresponding type where all potential
/// inner fields are optional.
/// In detail this means that an `Option<T::Optioned>` should allow for every combination
/// of itself being set as well as just partial subfields of itself being set.
/// Hence, for types without inner structure like `i32` the `Optioned` type will resolve to itself,
/// as e.g. `Option<i32>` already expresses the needed granularity.
pub trait Optionable {
    type Optioned;
}

/// This impl looks unintuitive. Keep in mind that `Optioned` has to be interpreted in the context
/// that an `Option<T::Optioned>` should allow for every combination. So we don't need
/// to keep an 'inner' `Option` here.
impl<T: Optionable> Optionable for Option<T> {
    type Optioned = T::Optioned;
}

// Blanket implementation for references to `Optionalable` types.
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
    String,
);

impl<'a> Optionable for &'a str {
    type Optioned = Self;
}

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
    fn option() {
        let a: Option<i32> = Some(10);
        // check that the 'inner' `Option` is removed, for reasoning see impl doc for `Option`.
        #[allow(clippy::unnecessary_literal_unwrap)]
        let _: <Option<i32> as Optionable>::Optioned = a.unwrap();
    }

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
