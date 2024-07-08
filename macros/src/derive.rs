//! # Derive
//!
//! Contains the `#[derive(Error)]` part of pisserror.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;
use syn::{spanned::Spanned as _, DeriveInput, Item};

use crate::{
    display::fmt,
    error::{cause, description, source},
    util::create_path,
};

pub fn derive_error(input: DeriveInput) -> syn::Result<TokenStream2> {
    let input_span = input.span();
    let input_ident = input.ident.clone();

    // make sure we've been given an enum.
    let Item::Enum(item) = Item::from(input) else {
        // remind user to use an enum.
        return Err(syn::Error::new_spanned(
            input_ident,
            "You must use an `enum` when deriving types with `pisserror`.",
        ));
    };

    let after_span = item.brace_token.span.close();

    let name = item.ident;
    let variants = item.variants;

    // Rust's Error type is defined as: `Error: Debug + Display`. to satisfy
    // `Debug`, we need to make sure the enum implements it.
    //
    // let debug_check = ...; // TODO: do some fancy checks. maybe assertions with an internal type?
    //
    // HEY! the compiler already does this for us! a nice error message might be preferable, though!

    // create each impl...

    // error impl
    let source = source(after_span, variants.iter())?; // TODO: check after_span
    let description = description();
    let cause = cause();

    // display impl
    let fmt = fmt(after_span, variants.iter(), name.clone())?;

    let error_path = create_path(input_span, &["std", "error", "Error"]);
    let display_path = create_path(input_span, &["core", "fmt", "Display"]);

    // put all those together!
    let impl_block = quote_spanned! {after_span=>
        use #error_path;

        #[automatically_derived]
        impl #error_path for #name {
            #source
            #description
            #cause
        }

        #[automatically_derived]
        impl #display_path for #name {
            #fmt
        }
    };

    Ok(impl_block)
}
