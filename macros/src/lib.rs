//! # Macros
//!
//! Macros for `pisserror`. These are re-exported by `pisserror` itself.
//!
//! In a fair world, this would be done with [reflection](https://t.co/vTvZ8FVEak).

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod derive;
pub(crate) mod display;
pub(crate) mod error;
mod util;

/// Derives `core::error::Error` from special syntax.
#[proc_macro_derive(Error, attributes(error, from))]
pub fn derive_error(input: TokenStream) -> TokenStream {
    // FIXME: all derives must be in the root module for some reason...
    let input = parse_macro_input!(input as DeriveInput);

    match derive::derive_error(input) {
        Ok(k) => k,
        Err(e) => e.into_compile_error(),
    }
    .into()
}
