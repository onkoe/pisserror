//! # Display
//!
//! Implements the `Display` trait for the user's error type.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::parser::{attr::ErrorAttribute, UserEnum};

impl UserEnum {
    /// The `Display` trait's `fmt` method.
    ///
    /// Let's check up on the result with some tests.
    /**

    ```compile_fail
    #[derive(Debug, Error)]
    enum Transparent {
        #[error(transparent)]
        VariantWithoutFrom,
    }
    ```

     */
    pub(crate) fn fmt(&self) -> TokenStream2 {
        let match_arms = if self.variants().is_empty() {
            // if there are no variants, add a catch-all arm.
            // (this is because of the reference match rule)
            vec![quote! { _ => Ok(()), }]
        } else {
            self.variants()
                .iter()
                .map(|v| {
                    let match_head = v.filled_match_head(self.ident());

                    // make the match arm
                    match v.error_attribute {
                        ErrorAttribute::Stringy(ref format_args_str) => {
                            quote! { #match_head => {f.write_str(format!(#format_args_str).as_str())} }
                        }

                        ErrorAttribute::Transparent => {
                            // use our `#[from]` field. b/c we MUST have one.
                            let from_field_ident = v
                                .from_attribute
                                .clone()
                                .expect("a `transparent` variant will have a `#[from]` field.")
                                .ident;

                            // check if we even have an ident
                            let format_args_str = from_field_ident.map_or_else(|| quote!(&_0.to_string()), |ident| quote!(&#ident.to_string()));
                            quote! { #match_head => { f.write_str(#format_args_str) }}
                        }
                    }
                })
                .collect()
        };

        let fmt_error = if cfg!(feature = "std") {
            quote!(std::fmt::Error)
        } else {
            quote!(core::fmt::Error)
        };

        quote! {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), #fmt_error> {
                match *self {
                    #(#match_arms),*
                }
            }
        }
    }
}
