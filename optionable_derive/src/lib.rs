//! # `optionable_derive`
//!
//! This crate is on its own not really usable. It is re-exported together with the required
//! traits in the [optionable](https://docs.rs/optionable/) crate. The relevant docs can be also found there.
//!
//! This is only a separate crate as derive macros have to be a separate crate.
use proc_macro::TokenStream;
use std::fmt;
mod derive;

/// Derive macro to derive the `Optionable` trait for structs/enums recursively. All non-required
/// fields have to implement the `Optionable` trait. This trait is already implemented by this library
/// for many primitive types, wrapper and container types.
///
/// ### Type-level attributes (on the struct/enum level)
/// - **`derive`**: Allows to specify derive attributes that should be attached to the generate optioned struct/enum.
///   Example:
///   ```rust,ignore
///   #[derive(optionable)]
///   #[optionable(derive(Deserialize, Serialize))]
///   struct MyStruct{}
///   ```
/// - **`suffix`**: The name of the generated optioned struct/enum will be `<original><suffix>` with suffix
///   defaulting to `"Opt"`. The suffix value can be adjusted via e.g. `#[optionable(suffix="Ac")]`.
///   Example:
///   ```rust,ignore
///   #[derive(optionable)]
///   #[optionable(suffix="Ac")]
///   struct MyStruct{}
///   ```
/// ### Field-level attributes (for structs and struct-typed enum variants)
/// - **`required`**: The annotated field will be kept as is and won't be transformed into some optional variant
///   for the derived optioned Struct.
///   Example:
///   ```rust,ignore
///   #[derive(optionable)]
///   struct MyStruct{
///     street: String; // will be an `Option<String>` in the derived `MyStructOpt`.
///     #[optionable(required)]
///     number: u32; // will also be a u32 in the derived `MyStructOpt`.
///   }
///   ```
#[proc_macro_derive(Optionable, attributes(optionable))]
pub fn derive_optionable(input: TokenStream) -> TokenStream {
    derive::derive_optionable(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// error just prepares an error message that references the source span
pub(crate) fn error<S: AsRef<str> + fmt::Display, T>(msg: S) -> syn::Result<T> {
    Err(syn::Error::new(proc_macro2::Span::call_site(), msg))
}
