use crate::error;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::default::Default;
use syn::{
    parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Type, TypePath, WhereClause,
    WherePredicate,
};

/// Derives the `Optionable`-trait from the main `optional_struct_recursive`-library.
/// Limited to structs atm.
/// todo: expand to e.g. enums
pub(crate) fn derive_optionable(input: TokenStream) -> syn::Result<TokenStream> {
    let mut input = syn::parse2::<DeriveInput>(input)?;
    patch_where_clause_bounds(&mut input.generics);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    if let Data::Struct(s) = input.data {
        let struct_name_opt = Ident::new(&(input.ident.to_string() + "Opt"), input.ident.span());
        let struct_name = &input.ident;
        let struct_fields = match s.fields {
            Fields::Named(f) => {
                let fields=f
                    .named
                    .into_iter()
                    .map(|f| (f.ident, f.ty))
                    .map(|(ident, ty)| quote! {#ident: Option<<#ty as  optional_struct_recursive::Optionable>::Optioned>,})
                    .collect::<Vec<_>>();
                quote!({
                    #(#fields)*
                })
            }
            Fields::Unnamed(f) => {
                let fields=f
                    .unnamed
                    .into_iter()
                    .map(|f| quote! {Option<<#f as  optional_struct_recursive::Optionable>::Optioned>,})
                    .collect::<Vec<_>>();
                quote!((
                    #(#fields)*
                );)
            }
            Fields::Unit => return error("#[derive(Optionable) not supported for unit structs"),
        };
        Ok(quote! {
            #[automatically_derived]
            struct  #struct_name_opt #impl_generics #where_clause #struct_fields

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
        error("#[derive(Optionable)] only supports structs")
    }
}

/// Adjusts the where clause to add the `Optionable` type bounds.
/// Basically the original where clause with a type bound to `Optionable` added
/// for every generic type parameter.
fn patch_where_clause_bounds(generics: &mut Generics) -> () {
    let where_clause = generics.where_clause.get_or_insert_with(|| WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });
    generics.params.iter().for_each(|param| {
        if let GenericParam::Type(type_param) = param {
            let ident = &type_param.ident;
            for pred in where_clause.predicates.iter_mut() {
                if let WherePredicate::Type(pred_ty) = pred
                    && let Type::Path(TypePath { qself: None, path }) = &pred_ty.bounded_ty
                    && path.is_ident(ident)
                {
                    // found an existing type bound for the given ident (e.g. `T`), add our `Optionable` bound
                    pred_ty
                        .bounds
                        .push(parse_quote!(optional_struct_recursive::Optionable));
                    return;
                }
            }
            // no type bound found, create a new one
            where_clause
                .predicates
                .push(parse_quote!(#ident: optional_struct_recursive::Optionable));
        }
    });
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
            // named struct fields
            TestCase {
                input: quote! {
                #[derive(Optionable)]
                    struct DeriveExample {
                        name: String,
                        surname: String,
                    }
                },
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
            // unnamed struct fields
            TestCase {
                input: quote! {
                    #[derive(Optionable)]
                    struct DeriveExample(String, i32);
                },
                output: quote! {
                    #[automatically_derived]
                    struct DeriveExampleOpt(
                        Option<<String as optional_struct_recursive::Optionable>::Optioned>,
                        Option<<i32 as optional_struct_recursive::Optionable>::Optioned>,
                    );

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
            // named struct fields with generics
            TestCase {
                input: quote! {
                    #[derive(Optionable)]
                    struct DeriveExample<T, T2: Serialize> where T: DeserializeOwned {
                        output: T,
                        input: T2,
                    }
                },
                output: quote! {
                    #[automatically_derived]
                    struct DeriveExampleOpt<T, T2: Serialize>
                        where T: DeserializeOwned + optional_struct_recursive::Optionable,
                              T2: optional_struct_recursive::Optionable {
                        output: Option<<T as optional_struct_recursive::Optionable>::Optioned>,
                        input: Option<<T2 as optional_struct_recursive::Optionable>::Optioned>,
                    }

                    #[automatically_derived]
                    impl<T, T2: Serialize> optional_struct_recursive::Optionable for DeriveExample<T, T2>
                        where T: DeserializeOwned + optional_struct_recursive::Optionable,
                              T2: optional_struct_recursive::Optionable  {
                        type Optioned = DeriveExampleOpt<T,T2>;
                    }

                    #[automatically_derived]
                    impl<T, T2: Serialize> optional_struct_recursive::Optionable for DeriveExampleOpt<T, T2>
                        where T: DeserializeOwned + optional_struct_recursive::Optionable,
                              T2: optional_struct_recursive::Optionable  {
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
