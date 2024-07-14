//! # Macros
//!
//! Macros for `pisserror`. These are re-exported by `pisserror` itself.
//!
//! In a fair world, this would be done with [reflection](https://t.co/vTvZ8FVEak).

use parse_enum::UserEnum;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod parse_enum;
mod traits;
mod util;

/// Derives `core::error::Error` from special syntax.
#[proc_macro_derive(Error, attributes(error, from))]
pub fn derive_error(input: TokenStream) -> TokenStream {
    // FIXME: all derives must be in the root module for some reason...
    let input = parse_macro_input!(input as DeriveInput);

    let user_enum = UserEnum::new(input);

    match traits::derive_error(input) {
        Ok(k) => k,
        Err(e) => e.into_compile_error(),
    }
    .into()
}
