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
    let from_attr = create_path(span, &["from"]);
    let mut list = Vec::with_capacity(variants.len());

    for variant in variants {
        // each variant can have one `#[from]` field
        let mut already_found_from_attr = false; // whether or not we found one
        let mut from_type = None; // the type if we did

        // look over each variant's field's attrs for the `#[from]` annotation!
        for field in &variant.fields {
            for attr in &field.attrs {
                if attr.path() == &from_attr {
                    // look if some other field has it. if so, get pissed
                    if already_found_from_attr {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "You may only have one `#[from]` attribute per variant.",
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
