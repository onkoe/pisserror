//! # Error
//!
//! Implements the `Error` trait for the user's error type.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, Ident, Variant};

use crate::{traits::from, util};

/// Parses the user's enum's variants to check for any internal `#[from]`
/// attributes, then generates code that matches on any given error variant.
///
/// # Attribute Rules
///
/// The `#[from]` attribute assumes it's only used once per variant.
///
/// As such, the following code shouldn't compile:
///
/** ```compile_fail
use macros::Error;
use std::error::Error;

#[derive(Debug, Error)]
enum SomeError {
    // you can't have two `#[from]` attrs on one variant!
    // note: this could actually be a cool feature if done right
    #[error("hello")]
    TwoAttrsOneField(#[from] std::io::Error, #[from] std::fmt::Error),
}
``` */
pub fn source(
    variants: &Punctuated<Variant, Comma>,
    variant_froms: &[&Variant],
    enum_name: &Ident,
) -> syn::Result<TokenStream2> {
    // store each variant's match arm, if it's even there!
    let match_arms = variants.iter().map(|v| {
        let is_from = variant_froms.contains(&v);

        match is_from {
            true => {
                // do some parsing nonsense
                let match_head = util::variant::make_variant_path(enum_name, &v.ident);

                match &v.fields {
                    syn::Fields::Named(_) => {
                        // get the identifier for the contained error
                        let container_err_ident = from::from_variants_identifer(v);

                        quote! {
                            #match_head {ref #container_err_ident} => Some(#container_err_ident)
                        }
                    }
                    syn::Fields::Unnamed(_) => quote! {
                        #match_head(ref e) => Some(e)
                    },
                    syn::Fields::Unit => quote! {
                        #match_head => None
                    },
                }
            }
            false => {
                // smooth sailing, baby!
                let match_head = util::variant::make_match_head(enum_name, v);

                quote! {
                    #match_head => None
                }
            }
        }
    });

    Ok(quote! {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match *self {
                #(#match_arms),*
            }
        }
    })
}

/// The method this generates is deprecated in favor of `Display`/`ToString`
/// on Error types, so we can safely return an empty string slice.
pub fn description() -> TokenStream2 {
    // TODO: consider using `Display` instead? check with other libraries b4.

    quote! {
        fn description(&self) -> &str {
            &""
        }
    }
}

/// The empty "cause" of the error. Now deprecated in favor of `source`, which
/// has the 'static bound.
///
/// As such, the method it generates always returns None.
pub fn cause() -> TokenStream2 {
    quote! {
        fn cause(&self) -> Option<&dyn Error> {
            None
        }
    }
}
