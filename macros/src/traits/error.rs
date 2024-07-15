//! # Error
//!
//! Implements the `Error` trait for the user's error type.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::parser::{field::WrappedFields, UserEnum};

impl UserEnum {
    /// The `Error` trait's `source` method.
    pub fn source(&self) -> TokenStream2 {
        let match_arms = self.variants().iter().map(|v| {
            match &v.from_attribute {
                // aw crap, we're an attribute. let's make some custom match arms...
                Some(info) => {
                    let variant_path = v.variant_path(self.ident());
                    let from_ident = &info.ident;

                    match v.fields {
                        WrappedFields::Named(_) => {
                            quote! { #variant_path { ref #from_ident} => Some(#from_ident)}
                        }
                        WrappedFields::Unnamed(_) => quote! { #variant_path(ref e) => Some(e) },
                        WrappedFields::Unit => {
                            unreachable!("unit enums cannot have a #[from] field")
                        }
                    }
                }

                // no #[from], so just give it None
                None => {
                    let left_side = v.match_head(self.ident());
                    quote! {#left_side => None}
                }
            }
        });

        quote! {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                match *self {
                    #(#match_arms),*
                }
            }
        }
    }

    /// The `Error` trait's `descritpion` method.
    ///
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

    /// The `Error` trait's `cause` method. Now deprecated in favor of
    /// `source`, which has the 'static bound.
    ///
    /// As such, this code always returns None.
    pub fn cause() -> TokenStream2 {
        quote! {
            fn cause(&self) -> Option<&dyn Error> {
                None
            }
        }
    }
}
