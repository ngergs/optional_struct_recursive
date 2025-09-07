use proc_macro::TokenStream;
use std::fmt;
mod derive;

/// Derives the `Optionable` trait.
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
