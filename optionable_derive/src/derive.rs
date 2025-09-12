use crate::error;
use darling::util::PathList;
use darling::{FromAttributes, FromDeriveInput};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::default::Default;
use syn::punctuated::Punctuated;
use syn::token::Where;
use syn::{
    parse_quote, Attribute, Data, DeriveInput, Fields, GenericParam, Generics, LitStr, Path, Type,
    TypePath, WhereClause, WherePredicate,
};

const HELPER_IDENT: &str = "optionable";
const ERR_MSG_HELPER_ATTR_ENUM_VARIANTS: &str =
    "#[optionable] helper attributes not supported on enum variant level.";

#[derive(FromDeriveInput)]
#[darling(attributes(optionable))]
/// Helper attributes on the type definition level (attached to the `struct` or `enum` itself).
struct TypeHelperAttributes {
    derive: Option<PathList>,
    #[darling(default=default_suffix)]
    suffix: LitStr,
}

#[derive(FromAttributes)]
#[darling(attributes(optionable))]
/// Helper attributes on the type definition level (attached to the `struct` or `enum` itself).
struct FieldHelperAttributes {
    required: Option<()>,
}

fn default_suffix() -> LitStr {
    LitStr::new("Opt", Span::call_site())
}

/// Derives the `Optionable`-trait from the main `optionable`-library.
pub(crate) fn derive_optionable(input: TokenStream) -> syn::Result<TokenStream> {
    let mut input = syn::parse2::<DeriveInput>(input)?;
    let attrs = TypeHelperAttributes::from_derive_input(&input)?;
    let vis = input.vis;
    let type_ident_opt = Ident::new(
        &(input.ident.to_string() + &attrs.suffix.value()),
        input.ident.span(),
    );
    let type_ident = &input.ident;

    patch_where_clause_bounds(&mut input.generics);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // the impl statements are actually independent of deriving
    // the relevant associated type #type_ident_opt referenced by them
    let impls = quote! {
        #[automatically_derived]
        impl #impl_generics ::optionable::Optionable for #type_ident #ty_generics #where_clause {
            type Optioned = #type_ident_opt #ty_generics;
        }

        #[automatically_derived]
        impl #impl_generics ::optionable::Optionable for #type_ident_opt #ty_generics #where_clause {
            type Optioned = #type_ident_opt #ty_generics;
        }
    };

    // now we have to derive the actual implementation of #type_ident_opt
    // and add the #impl from above
    let derives = attrs.derive.unwrap_or_default();
    let skip_optionable_if_serde_serialize = derives
        .iter()
        .any(is_serialize)
        .then(|| quote!(#[serde(skip_serializing_if = "Option::is_none")]));
    let derives = derives
        .iter()
        .map(ToTokens::to_token_stream)
        .collect::<Vec<_>>();
    let derives = (!derives.is_empty()).then(|| quote! {#[derive(#(#derives),*)]});
    match input.data {
        Data::Struct(s) => {
            let unnamed_struct_semicolon = (if let Fields::Unnamed(_) = &s.fields {
                Some(quote! {;})
            } else {
                None
            })
            .to_token_stream();
            let fields = optioned_fields(s.fields, skip_optionable_if_serde_serialize.as_ref())?;

            Ok(quote! {
                #[automatically_derived]
                #derives
                #vis struct #type_ident_opt #impl_generics #where_clause #fields #unnamed_struct_semicolon

                #impls
            })
        }
        Data::Enum(e) => {
            let variants = e
                .variants
                .into_iter()
                .map(|v| {
                    error_on_helper_attributes(&v.attrs, ERR_MSG_HELPER_ATTR_ENUM_VARIANTS)?;
                    let fields =
                        optioned_fields(v.fields, skip_optionable_if_serde_serialize.as_ref())?;
                    Ok::<_, syn::Error>((v.ident, fields))
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .map(|(ident, fields)| quote!( #ident #fields ))
                .collect::<Vec<_>>();
            Ok(quote!(
                #[automatically_derived]
                #derives
                #vis enum #type_ident_opt #impl_generics #where_clause {
                    #(#variants),*
                }
                #impls
            ))
        }
        Data::Union(_) => error("#[derive(Optionable)] not supported for unit structs"),
    }
}

/// Goes through the attributes, filters for our [`HELPER_IDENT`] helper-attribute identifier
/// and reports an error if anything is found.
fn error_on_helper_attributes(attrs: &[Attribute], err_msg: &'static str) -> syn::Result<()> {
    if attrs
        .iter()
        .filter(|attr| {
            println!("{}", attr.path().to_token_stream());
            attr.path().is_ident(HELPER_IDENT)
        })
        .collect::<Vec<_>>()
        .is_empty()
    {
        Ok(())
    } else {
        error(err_msg)
    }
}

/// Returns a tokenstream for the fields of the optioned object (struct/enum variants).
/// The returned tokenstream will be of the form `{...}` for named fields and `(...)` for unnamed fields.
/// Does not include any leading `struct/enum` keywords or any trailing `;`.
fn optioned_fields(
    fields: Fields,
    serde_attributes: Option<&TokenStream>,
) -> syn::Result<TokenStream> {
    Ok(match fields {
        Fields::Named(f) => {
            let fields = f
                .named
                .into_iter()
                .map(|f| {
                    let (vis, ident, ty) = (f.vis, f.ident, f.ty);
                    let attrs = FieldHelperAttributes::from_attributes(&f.attrs)?;
                    Ok::<_, syn::Error>(if attrs.required.is_some() {
                        quote! {#vis #ident: #ty}
                    } else if is_option(&ty) {
                        // Type is already an Option, no need to add an outer one
                        quote! {#serde_attributes #vis #ident: <#ty as ::optionable::Optionable>::Optioned}
                    } else {
                        quote! {#serde_attributes #vis #ident: Option<<#ty as  ::optionable::Optionable>::Optioned>}
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            quote!({
                #(#fields),*
            })
        }
        Fields::Unnamed(f) => {
            let fields = f
                .unnamed
                .into_iter()
                .map(|f| {
                    let (vis, ty) = (f.vis, f.ty);
                    let attrs = FieldHelperAttributes::from_attributes(&f.attrs)?;
                    Ok::<_, syn::Error>(if attrs.required.is_some() {
                        quote! {#vis #ty}
                    } else {
                        quote! {#serde_attributes #vis Option<<#ty as  ::optionable::Optionable>::Optioned>}
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            quote!((
                #(#fields),*
            ))
        }
        Fields::Unit => quote!(),
    })
}

/// Checks whether this type identifier is a `std::option::Option` or a shortened variant of it.
fn is_option(ty: &Type) -> bool {
    if let Type::Path(TypePath {
        qself: _qself,
        path,
    }) = &ty
        && {
            let segments = &path.segments;
            (segments.len() == 1 && segments[0].ident == "Option")
                || (segments.len() == 2
                    && segments[0].ident == "option"
                    && segments[1].ident == "Option")
                || (segments.len() == 3
                    && segments[0].ident == "std"
                    && segments[1].ident == "option"
                    && segments[2].ident == "Option")
        }
    {
        println!("{:?}", path.segments);
        true
    } else {
        false
    }
}

/// Checks whether this path is `serde::Serialize` or a shortened version of it.
fn is_serialize(path: &Path) -> bool {
    path.is_ident("Serialize") || {
        let segments = &path.segments;
        segments.len() == 2 && segments[0].ident == "serde" && segments[1].ident == "Serialize"
    }
}

/// Adjusts the where clause to add the `Optionable` type bounds.
/// Basically the original where clause with a type bound to `Optionable` added
/// for every generic type parameter.
fn patch_where_clause_bounds(generics: &mut Generics) {
    let where_clause = generics.where_clause.get_or_insert_with(|| WhereClause {
        where_token: Where::default(),
        predicates: Punctuated::default(),
    });
    generics.params.iter().for_each(|param| {
        if let GenericParam::Type(type_param) = param {
            let ident = &type_param.ident;
            for pred in &mut where_clause.predicates {
                if let WherePredicate::Type(pred_ty) = pred
                    && let Type::Path(TypePath { qself: None, path }) = &pred_ty.bounded_ty
                    && path.is_ident(ident)
                {
                    // found an existing type bound for the given ident (e.g. `T`), add our `Optionable` bound
                    pred_ty.bounds.push(parse_quote!(::optionable::Optionable));
                    return;
                }
            }
            // no type bound found, create a new one
            where_clause
                .predicates
                .push(parse_quote!(#ident: ::optionable::Optionable));
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
    #[allow(clippy::too_many_lines)]
    fn test_optionable() {
        let tcs = vec![
            // named struct fields
            TestCase {
                input: quote! {
                    #[derive(Optionable)]
                    struct DeriveExample {
                        name: String,
                        pub surname: String,
                    }
                },
                output: quote! {
                    #[automatically_derived]
                    struct DeriveExampleOpt {
                        name: Option<<String as ::optionable::Optionable>::Optioned>,
                        pub surname: Option<<String as ::optionable::Optionable>::Optioned>
                    }

                    #[automatically_derived]
                    impl ::optionable::Optionable for DeriveExample {
                        type Optioned = DeriveExampleOpt;
                    }

                    #[automatically_derived]
                    impl ::optionable::Optionable for DeriveExampleOpt {
                        type Optioned = DeriveExampleOpt;
                    }
                },
            },
            // named struct fields with required fields
            TestCase {
                input: quote! {
                    #[derive(Optionable)]
                    struct DeriveExample {
                        name: String,
                        #[optionable(required)]
                        pub surname: String,
                    }
                },
                output: quote! {
                    #[automatically_derived]
                    struct DeriveExampleOpt {
                        name: Option<<String as ::optionable::Optionable>::Optioned>,
                        pub surname: String
                    }

                    #[automatically_derived]
                    impl ::optionable::Optionable for DeriveExample {
                        type Optioned = DeriveExampleOpt;
                    }

                    #[automatically_derived]
                    impl ::optionable::Optionable for DeriveExampleOpt {
                        type Optioned = DeriveExampleOpt;
                    }
                },
            },
            // named struct fields with forwarded derives and Serialize annotations
            TestCase {
                input: quote! {
                    #[derive(Optionable)]
                    #[optionable(derive(Deserialize,Serialize),suffix="Ac")]
                    struct DeriveExample {
                        name: String,
                        middle_name: Option<String>,
                        surname: String,
                    }
                },
                output: quote! {
                    #[automatically_derived]
                    #[derive(Deserialize, Serialize)]
                    struct DeriveExampleAc {
                        #[serde(skip_serializing_if = "Option::is_none")]
                        name: Option<<String as ::optionable::Optionable>::Optioned>,
                        #[serde(skip_serializing_if = "Option::is_none")]
                        middle_name: <Option<String> as ::optionable::Optionable>::Optioned,
                        #[serde(skip_serializing_if = "Option::is_none")]
                        surname: Option<<String as ::optionable::Optionable>::Optioned>
                    }

                    #[automatically_derived]
                    impl ::optionable::Optionable for DeriveExample {
                        type Optioned = DeriveExampleAc;
                    }

                    #[automatically_derived]
                    impl ::optionable::Optionable for DeriveExampleAc {
                        type Optioned = DeriveExampleAc;
                    }
                },
            },
            // named struct fields with forwarded derives and Serialize annotations (full path variant)
            TestCase {
                input: quote! {
                    #[derive(Optionable)]
                    #[optionable(derive(serde::Deserialize,serde::Serialize),suffix="Ac")]
                    struct DeriveExample {
                        name: String,
                        surname: String,
                    }
                },
                output: quote! {
                    #[automatically_derived]
                    #[derive(serde::Deserialize, serde::Serialize)]
                    struct DeriveExampleAc {
                        #[serde(skip_serializing_if = "Option::is_none")]
                        name: Option<<String as ::optionable::Optionable>::Optioned>,
                        #[serde(skip_serializing_if = "Option::is_none")]
                        surname: Option<<String as ::optionable::Optionable>::Optioned>
                    }

                    #[automatically_derived]
                    impl ::optionable::Optionable for DeriveExample {
                        type Optioned = DeriveExampleAc;
                    }

                    #[automatically_derived]
                    impl ::optionable::Optionable for DeriveExampleAc {
                        type Optioned = DeriveExampleAc;
                    }
                },
            },
            // unnamed struct fields
            TestCase {
                input: quote! {
                    #[derive(Optionable)]
                    struct DeriveExample(pub String, i32);
                },
                output: quote! {
                    #[automatically_derived]
                    struct DeriveExampleOpt(
                        pub Option<<String as ::optionable::Optionable>::Optioned>,
                        Option<<i32 as ::optionable::Optionable>::Optioned>
                    );

                    #[automatically_derived]
                    impl ::optionable::Optionable for DeriveExample {
                        type Optioned = DeriveExampleOpt;
                    }

                    #[automatically_derived]
                    impl ::optionable::Optionable for DeriveExampleOpt {
                        type Optioned = DeriveExampleOpt;
                    }
                },
            },
            // unnamed struct fields with required
            TestCase {
                input: quote! {
                    #[derive(Optionable)]
                    struct DeriveExample(pub String, #[optionable(required)] i32);
                },
                output: quote! {
                    #[automatically_derived]
                    struct DeriveExampleOpt(
                        pub Option<<String as ::optionable::Optionable>::Optioned>,
                        i32
                    );

                    #[automatically_derived]
                    impl ::optionable::Optionable for DeriveExample {
                        type Optioned = DeriveExampleOpt;
                    }

                    #[automatically_derived]
                    impl ::optionable::Optionable for DeriveExampleOpt {
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
                        where T: DeserializeOwned + ::optionable::Optionable,
                              T2: ::optionable::Optionable {
                        output: Option<<T as ::optionable::Optionable>::Optioned>,
                        input: Option<<T2 as ::optionable::Optionable>::Optioned>
                    }

                    #[automatically_derived]
                    impl<T, T2: Serialize> ::optionable::Optionable for DeriveExample<T, T2>
                        where T: DeserializeOwned + ::optionable::Optionable,
                              T2: ::optionable::Optionable  {
                        type Optioned = DeriveExampleOpt<T,T2>;
                    }

                    #[automatically_derived]
                    impl<T, T2: Serialize> ::optionable::Optionable for DeriveExampleOpt<T, T2>
                        where T: DeserializeOwned + ::optionable::Optionable,
                              T2: ::optionable::Optionable  {
                        type Optioned = DeriveExampleOpt<T,T2>;
                    }
                },
            },
            TestCase {
                input: quote! {
                    #[derive(Optionable)]
                    enum DeriveExample {
                        Unit,
                        Plain(String),
                        Address{street: String, number: u32},
                        Address2(String,u32),
                    }
                },
                output: quote! {
                    # [automatically_derived]
                    enum DeriveExampleOpt {
                        Unit,
                        Plain( Option<<String as ::optionable::Optionable>::Optioned> ),
                        Address{ street: Option<< String as ::optionable::Optionable>::Optioned>, number:Option<<u32 as ::optionable::Optionable>::Optioned> },
                        Address2( Option<<String as ::optionable::Optionable>::Optioned>, Option<<u32 as ::optionable::Optionable>::Optioned> )
                    }

                    #[automatically_derived]
                    impl ::optionable::Optionable for DeriveExample {
                        type Optioned = DeriveExampleOpt;
                    }

                    #[automatically_derived]
                    impl ::optionable::Optionable for DeriveExampleOpt {
                        type Optioned = DeriveExampleOpt;
                    }
                },
            },
        ];
        for tc in tcs {
            let output = derive_optionable(tc.input).unwrap();
            println!("{output}");
            assert_eq!(tc.output.to_string(), output.to_string());
        }
    }
}
