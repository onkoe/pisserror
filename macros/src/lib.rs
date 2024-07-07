//! # Macros
//!
//! Macros for `pisserror`. These are re-exported by `pisserror` itself.
//!
//! In a fair world, this would be done with [reflection](https://t.co/vTvZ8FVEak).

use proc_macro::TokenStream;

mod derive;
pub(crate) mod display;
pub(crate) mod error;
mod util;

/// Derives `core::error::Error` from special syntax.
#[proc_macro_derive(Error)]
pub fn derive_error(input: TokenStream) -> TokenStream {
    // FIXME: all derives must be in the root module for some reason...
    derive::derive_error(input)
}
