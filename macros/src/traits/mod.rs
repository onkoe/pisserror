//! # Traits
//!
//! Contains the `#[derive(Error)]` part of pisserror. This module holds and
//! uses the implementation for each trait.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;
use syn::{spanned::Spanned as _, DeriveInput, Item, Variant};

use crate::util;

mod display;
mod error;
mod from;

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

    // check `From` impl eligibility
    let variants_with_froms = from::fields_with_from_attrs(input_span, &variants)?;

    // for each variant, make them a from block
    let froms = variants_with_froms
        .iter()
        .map(|(v, t)| from::from(&name, v, t));

    // make all Error impl fns...
    let source = error::source(
        &variants,
        &variants_with_froms
            .iter()
            .map(|(v, _)| *v)
            .collect::<Vec<&Variant>>(),
        &name,
    )?; // TODO: check after_span
    let description = error::description();
    let cause = error::cause();

    // ...and all Display impl fns
    let fmt = display::fmt(after_span, &variants, &name)?;

    let error_path = util::create_path(input_span, &["std", "error", "Error"]);
    let display_path = util::create_path(input_span, &["core", "fmt", "Display"]);

    // put all those together!
    let impl_block = quote_spanned! {after_span=>
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

        #(#froms)*
    };

    Ok(impl_block)
}
