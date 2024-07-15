//! # Display
//!
//! Implements the `Display` trait for the user's error type.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::parser::UserEnum;

impl UserEnum {
    /// The `Display` trait's `fmt` method.
    pub fn fmt(&self) -> TokenStream2 {
        let match_arms = if self.variants().is_empty() {
            // if there are no variants, add a catch-all arm.
            // (this is because of the reference match rule)
            vec![quote! { _ => Ok(()), }]
        } else {
            self.variants()
                .iter()
                .map(|v| {
                    let match_head = v.filled_match_head(self.ident());
                    let tokens = &v.error_attribute.format_string;
                    quote! { #match_head => {f.write_str(format!(#tokens).as_str())} }
                })
                .collect()
        };

        quote! {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                match self {
                    #(#match_arms),*
                }
            }
        }
    }
}
