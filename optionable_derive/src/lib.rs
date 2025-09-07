use proc_macro::TokenStream;
use std::fmt;
mod derive;

/// Derives the `Optionable` trait.
///
/// If you want the `Optioned`-structure/enum to also derive additional traits
/// add on the struct/enum-eval `#[optionable(derive(<List of Derives))]`,
/// e.g. `#[optionable(derive(Deserialize,Serialize))]`.
///
/// By default, this will generate an "optioned" struct/enum with the name "<original>Opt".
/// If this causes naming collisions you can specify the suffix with `#[optionable(optioned_suffix="SuffixValue")]`.
///
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
