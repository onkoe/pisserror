//! # From
//!
//! Implements the `From` trait for the user's type.

use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use syn::{punctuated::Punctuated, token::Comma, Field, Ident, Type, Variant};

use crate::util::create_path;

/// Returns a `Vec<(&Variant, Option<&Type>)>`, showing which variants have
/// `#[from]`, attributes, and which errors those variants implement.
///
/// However, passes back an `Err` when the attribute's usages are wrong.
pub(crate) fn fields_with_from_attrs(
    span: Span2,
    variants: &Punctuated<Variant, Comma>,
) -> syn::Result<Vec<(&Variant, Option<&Type>)>> {
    let from_attr: syn::Path = create_path(span, &["from"]);
    let mut list = Vec::with_capacity(variants.len());

    for variant in variants {
        // each variant can have one `#[from]` field
        let mut already_found_from_attr = false; // whether or not we found one
        let mut from_type = None; // the type if we did

        // look over each variant's field's attrs for the `#[from]` annotation!
        for field in &variant.fields {
            for attr in &field.attrs {
                if attr.path() == &from_attr {
                    // look if some other field has it. if so, get pissed.
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
                    from_type = Some(&field.ty);
                }
            }
        }

        // add variant to vec alongside its `From<Type>`
        list.push((variant, from_type));
    }

    // return the list we made!
    Ok(list)
}
/// `Some(Err(TokenStream2))` when something is wrong with the user's `#[from]`
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
