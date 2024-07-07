//! # Util
//!
//! Some utilities to help out with writing these macros.

use proc_macro::{Span, TokenStream};
use quote::quote_spanned;

pub fn comp_time_err(span: Span, msg: &str) -> TokenStream {
    quote_spanned! {span.into()=>
        compile_error!(msg);
    }
    .into()
}
