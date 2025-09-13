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

use crate::optionable::Error;
#[doc(inline)]
pub use optionable_derive::Optionable;

pub mod optionable;

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

/// Helper methods to transform in and from optioned objects as well as merging.
/// Only available for sized types.
pub trait OptionableConvert: Sized + Optionable {
    /// Transforms this object into an optioned variant which all fields set.
    ///
    /// We cannot implement `Into` from the stdlib as we need to implement this
    /// for various stdlib primitives and containers.
    fn into_optioned(self) -> Self::Optioned;

    /// Try to build this full type from its optioned variant.
    ///
    /// We cannot implement `TryFrom` from the stdlib as we need to implement this
    /// for various stdlib primitives and containers.
    ///
    /// # Errors
    /// - If fields required by the full type are not set.
    fn try_from_optioned(value: Self::Optioned) -> Result<Self, Error>;
    /// Merge the optioned values into this full type. List-like types are overwritten if set in `other`.
    /// Maps are merged per key.
    ///
    /// # Errors
    /// - There are scenarios where the full type allows some missing fields but the optioned type
    ///   also does not hold enough subfields to constructs a full entry with the respective `try_from`.
    ///   An example would be a field with type `Option<T>` and value `None` for `self` and type `Option<T::Optioned>`
    ///   and `Some` value for `other`. The `T::try_from(T::Optioned)` can fail is fields are missing for this subfield.
    fn merge(&mut self, other: Self::Optioned) -> Result<(), Error>;
}
