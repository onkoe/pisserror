//! # Traits
//!
//! Contains the `#[derive(Error)]` part of pisserror. This module holds and
//! uses the implementation for each trait.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;

use crate::{parser::UserEnum, util};

mod display;
mod error;
mod from;

pub(crate) fn derive_error(user_enum: &UserEnum) -> syn::Result<TokenStream2> {
    // make a from block for each variant
    let froms = user_enum.from();

    // make all Error impl fns...
    let source = user_enum.source();
    let description = UserEnum::description();
    let cause = UserEnum::cause();

    // ...and all Display impl fns
    let fmt = user_enum.fmt();

    let error_path = if cfg!(feature = "std") {
        util::create_path(user_enum.span(), &["std", "error", "Error"])
    } else {
        util::create_path(user_enum.span(), &["core", "error", "Error"])
    };
    let display_path = util::create_path(user_enum.span(), &["core", "fmt", "Display"]);

    // some extra variables to make quote not scare me as much
    let enum_ident = user_enum.ident();
    let after_span = user_enum.after_span();

    // put all those together!
    let impl_block = quote_spanned! {after_span=>
        #[automatically_derived]
        impl #error_path for #enum_ident {
            #source
            #description
            #cause
        }

        #[automatically_derived]
        impl #display_path for #enum_ident {
            #fmt
        }

        #froms
    };

    Ok(impl_block)
}
