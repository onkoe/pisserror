//! # Macros
//!
//! Internal implemenation of `pisserror`. These are re-exported by `pisserror` itself.
//!
//! In a fair world, this would be done with [reflection](https://t.co/vTvZ8FVEak).

use parser::UserEnum;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

pub(crate) mod parser;
pub(crate) mod traits;
pub(crate) mod util;

/// Derives `core::error::Error` from special syntax.
#[proc_macro_derive(Error, attributes(error, from))]
pub fn derive_error(input: TokenStream) -> TokenStream {
    // FIXME: all derives must be in the root module for some reason...
    let synd_input = parse_macro_input!(input as DeriveInput);

    let user_enum = match UserEnum::new(synd_input) {
        Ok(ue) => ue,
        Err(e) => {
            return e.into_compile_error().into();
        }
    };

    match traits::derive_error(&user_enum) {
        Ok(ts) => ts,
        Err(e) => e.into_compile_error(),
    }
    .into()
}
