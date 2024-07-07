//! # Derive
//!
//! Contains the `#[derive(Error)]` part of pisserror.

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned as _, DeriveInput, Item};

pub fn derive_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input_span = input.span();

    // make sure we've been given an enum.
    if let Item::Enum(item) = Item::from(input) {
        // check each variant and get info on their `#[error(...)]` attribute.
        let name = item.ident;

        // ensure it implements `Debug`. this is one of the bounds of `core::error::Error`, so it's a good check to have...
        // let debug_check = ...; // TODO: do some fancy checks. maybe assertions with an internal type?

        // assemble the match arms
        let fn_source = {
            // TODO: check for any `from` attributes on variants
            quote! {
                fn source(&self) -> Option<&(dyn Error + 'static)> {
                    None
                }
            }
        };

        // these days, you implement Display instead. deprecated - leave blank.
        // TODO: consider calling Display/ToString instead of blank str slice ref?
        let fn_description = quote! {
            fn description(&self) -> &str {
                &""
            }
        };

        // previous version of "source" - deprecated, so leave blank.
        let fn_cause = quote! {
            fn cause(&self) -> Option<&dyn Error> {
                None
            }
        };

        // put all those together!
        let impl_block = quote_spanned! {input_span=>
            #[automatically_derived]
            impl core::error::Error for #name {
                #fn_source
                #fn_description
                #fn_cause

                compile_error!("TODO: Error is not yet implemented.")
            }

            #[automatically_derived]
            impl core::fmt::Display for #name {
                compile_error!("TODO: Display is not yet implemented.")
            }
        };

        impl_block.into()
    } else {
        // remind user to use an enum.
        quote_spanned! {input_span=>
            compile_error!("You must use an `enum` when deriving types with `pisserror`.");
        }
        .into()
    }
}

/// Rust's Error type is defined as: `Error: Debug + Display`. to satisfy
/// `Debug`, we need to make sure the enum implements it.
///
/// this function returns a token stream that contains a
pub fn check_enum_impl_debug(item: Item) -> TokenStream {
    todo!()
}
