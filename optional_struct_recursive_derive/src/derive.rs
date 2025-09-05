use crate::error;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Fields};

/// Derives the `Optionable`-trait from the main `optional_struct_recursive`-library.
/// Limited to structs atm.
/// todo: expand to e.g. enums
pub(crate) fn derive_optionable(input: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse2::<DeriveInput>(input)?;
    if let Data::Struct(s) = input.data {
        if let Fields::Named(fields) = s.fields {
            let struct_name_opt =
                Ident::new(&(input.ident.to_string() + "Opt"), input.ident.span());
            let struct_name = &input.ident;
            let fields = fields
                .named
                .into_iter()
                .map(|f| (f.ident, f.ty))
                .map(|(ident, ty)| quote! {#ident: Option<<#ty as  optional_struct_recursive::Optionable>::Optioned>,});
            let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
            Ok(quote! {
                #[automatically_derived]
                struct  #struct_name_opt #impl_generics #where_clause {
                    #(#fields)*
                }

                #[automatically_derived]
                impl #impl_generics optional_struct_recursive::Optionable for #struct_name #ty_generics #where_clause {
                    type Optioned = #struct_name_opt #ty_generics;
                }

                #[automatically_derived]
                impl #impl_generics optional_struct_recursive::Optionable for #struct_name_opt #ty_generics #where_clause {
                    type Optioned = #struct_name_opt #ty_generics;
                }
            })
        } else {
            error("#[derive(Optionable)] only supports structs with named fields")
        }
    } else {
        error("#[derive(Optionable)] only supports structs with named fields")
    }
}

#[cfg(test)]
mod tests {
    use crate::derive::derive_optionable;
    use proc_macro2::TokenStream;
    use quote::quote;

    struct TestCase {
        input: TokenStream,
        output: TokenStream,
    }

    #[test]
    fn test_optionable() {
        let tcs = vec![
            TestCase {
                input: quote! {
                #[derive(Optionable)]
                struct DeriveExample {
                    name: String,
                    surname: String,
                }},
                output: quote! {
                    #[automatically_derived]
                    struct DeriveExampleOpt {
                        name: Option<<String as optional_struct_recursive::Optionable>::Optioned>,
                        surname: Option<<String as optional_struct_recursive::Optionable>::Optioned>,
                    }

                    #[automatically_derived]
                    impl optional_struct_recursive::Optionable for DeriveExample {
                        type Optioned = DeriveExampleOpt;
                    }

                    #[automatically_derived]
                    impl optional_struct_recursive::Optionable for DeriveExampleOpt {
                        type Optioned = DeriveExampleOpt;
                    }
                },
            },
            TestCase {
                input: quote! {
                #[derive(Optionable)]
                struct DeriveExample<T, T2: Serialize> where T: DeserializeOwned {
                    output: T,
                    input: T2,
                }},
                output: quote! {
                    #[automatically_derived]
                    struct DeriveExampleOpt<T, T2: Serialize> where T: DeserializeOwned {
                        output: Option<<T as optional_struct_recursive::Optionable>::Optioned>,
                        input: Option<<T2 as optional_struct_recursive::Optionable>::Optioned>,
                    }

                    #[automatically_derived]
                    impl<T, T2: Serialize> optional_struct_recursive::Optionable for DeriveExample<T, T2> where T: DeserializeOwned{
                        type Optioned = DeriveExampleOpt<T,T2>;
                    }

                    #[automatically_derived]
                    impl<T, T2: Serialize> optional_struct_recursive::Optionable for DeriveExampleOpt<T, T2> where T: DeserializeOwned{
                        type Optioned = DeriveExampleOpt<T,T2>;
                    }
                },
            },
        ];
        for tc in tcs {
            let output = derive_optionable(tc.input).unwrap();
            println!("{}", output.to_string());
            assert_eq!(tc.output.to_string(), output.to_string());
        }
    }
}
