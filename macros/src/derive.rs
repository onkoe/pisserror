//! # Derive
//!
//! Contains the `#[derive(Error)]` part of pisserror.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned as _, DeriveInput, Item, Variant};

pub fn derive_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input_span = input.span();

    // make sure we've been given an enum.
    if let Item::Enum(item) = Item::from(input) {
        let name = item.ident;
        let variants = item.variants;

        // Rust's Error type is defined as: `Error: Debug + Display`. to satisfy
        // `Debug`, we need to make sure the enum implements it.
        //
        // let debug_check = ...; // TODO: do some fancy checks. maybe assertions with an internal type?
        //
        // HEY! the compiler already does this for us! a nice error message might be preferable, though!

        // assemble the match arms
        let source = source(variants.iter());
        let description = description();
        let cause = cause();

        // previous version of "source" - deprecated, so leave blank.

        // put all those together!
        let impl_block = quote_spanned! {input_span=>
            // TODO: check each variant and get info on their `#[error(...)]` attribute.
            #[automatically_derived]
            impl std::error::Error for #name {
                #source
                #description
                #cause

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

/// err: Option<&(dyn std::error::Error + 'static)>
fn source<'a>(variants: impl Iterator<Item = &'a Variant>) -> TokenStream2 {
    // TODO: check for any `from` attributes on variants
    quote! {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            None
        }
    }
}

// these days, you implement Display instead. deprecated - leave blank.
// TODO: consider calling Display/ToString instead of blank str slice ref?
fn description() -> TokenStream2 {
    quote! {
        fn description(&self) -> &str {
            &""
        }
    }
}

fn cause() -> TokenStream2 {
    quote! {
        fn cause(&self) -> Option<&dyn Error> {
            None
        }
    }
}
