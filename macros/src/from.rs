//! # From
//!
//! Implements the `From` trait for the user's type.

use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::{punctuated::Punctuated, spanned::Spanned, token::Comma, Ident, Type, Variant};

use crate::util::create_path;

/// Returns a list of the variants who have `#[from]` attributes, alongside the
/// errors they implement.
///
/// However, passes back an `Err` when the attribute's usages are wrong.
pub(crate) fn fields_with_from_attrs(
    span: Span2,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<Vec<(&Variant, &Type)>> {
    let from_attr: syn::Path = create_path(span, &["from"]);
    let mut list = Vec::with_capacity(variants.len());

    for variant in variants {
        let mut already_found_from_attr = false; // whether or not we found one

        // look over each variant's field's attrs for the `#[from]` annotation!
        for field in &variant.fields {
            for attr in &field.attrs {
                if attr.path() == &from_attr {
                    // each variant can have one `#[from]` field
                    if already_found_from_attr {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "You may only have one `#[from]` attribute per variant.",
                        ));
                    }

                    // if we have `#[from]`, there can be no other fields on this variant
                    if variant.fields.len() > 1 {
                        return Err(syn::Error::new_spanned(
                            variant,
                            "A variant containing a field with the `#[from]` \
                            attribute must have only one field.\n
                            
                            Please see: \
                            https://github.com/onkoe/pisserror/issues/11#issuecomment-2215435824",
                        ));
                    }

                    // otherwise, remember the From<Type> we were given
                    already_found_from_attr = true;
                    // add variant to vec alongside its `From<Type>`
                    list.push((variant, &field.ty));
                }
            }
        }
    }

    // return the list we made
    Ok(list)
}

/// Gets the identifier for a `#[from]` variant's field.
///
/// ONLY PASS THIS FUNCTION A VARIANT WHOSE FIELD HAS A FROM ATTR.
/// TODO: make this take some custom enum type:
///       - NormalVariant(Variant)
///       - FromVariant(Variant, Type)
pub(crate) fn from_variants_identifer(variant: &Variant) -> Ident {
    match &variant.fields {
        syn::Fields::Unit | syn::Fields::Unnamed(_) => {
            unreachable!("we should never get a `#[from]` variant with no/unnamed fields")
        }
        syn::Fields::Named(n) => n
            .named
            .first()
            .expect("a variant with a field will always have a first field")
            .ident
            .clone()
            .expect("a named field will always be named"),
    }
}

/// Creates a `From<other::Error> for UserError` impl for the given variant
/// and field.
pub(crate) fn from(enum_name: &Ident, variant: &Variant, ty: &Type) -> TokenStream2 {
    // TODO: HEY! DO NOT USE `fields_with_from_attrs` IN HERE! MAKE `derive.rs`
    //       CALL + PASS IT IN! SAME WITH `error::source()`!!!

    // grab the necessary names
    let variant_name = &variant.ident;

    // let's decide which style to use during construction
    let style = match &variant.fields {
        syn::Fields::Named(n) => {
            let field_name = from_variants_identifer(variant);
            quote_spanned!(n.span()=> #enum_name::#variant_name {#field_name: value})
        }
        syn::Fields::Unnamed(un) => quote_spanned!(un.span()=> #enum_name::#variant_name(value)),
        syn::Fields::Unit => unreachable!(),
    };

    quote! {
        #[automatically_derived]
        impl core::convert::From<#ty> for #enum_name {
            fn from(value: #ty) -> #enum_name {
                #style
            }
        }
    }
}

#[cfg(test)]
mod tests {
    mod field_checking_tests {
        use crate::from::fields_with_from_attrs;
        use syn::{parse_quote, spanned::Spanned, ItemEnum};

        #[test]
        fn error_on_multiple_froms() {
            // make the enum
            let sauce: ItemEnum = parse_quote! {
                enum MyError {
                    // you may not have multiple from attributes, so this should fail
                    SomeVariant(#[from] std::io::Error, #[from] std::collection::TryReserveError)
                }
            };

            // it should have errors
            assert!(fields_with_from_attrs(sauce.span(), &sauce.variants).is_err());
        }

        #[test]
        fn works_with_many_variants() {
            // make the enum
            let sauce: ItemEnum = parse_quote! {
                // these variants all have one from - it should work great!
                enum MyError {
                    VariantOne(#[from] std::io::Error),
                    VariantTwo(#[from] std::collection::TryReserveError),
                    VariantThree {
                        #[from]
                        some_field: std::array::TryFromSliceError,
                    }
                }
            };

            // check it
            assert!(fields_with_from_attrs(sauce.span(), &sauce.variants).is_ok());
        }

        #[test]
        fn errs_with_multiple_fields() {
            // make the enum
            let sauce: ItemEnum = parse_quote! {
                enum MyError {
                    // if a variant's field has a `#[from]` attr, it MUST be the only field
                    StructLikeVariant {
                        #[from]
                        some_error_type: std::io::Error,
                        favorite_number: u32,
                    }
                }
            };

            assert!(fields_with_from_attrs(sauce.span(), &sauce.variants).is_err());
        }
    }

    mod from_trait_gen_tests {
        #[test]
        fn two_variants_one_error() {
            // two variants with the same From<Error> should fail
            // assert!(result.is_err());
        }
    }
}
