//! # Util
//!
//! Some utilities to help out with writing these macros.

use proc_macro::{Span, TokenStream};
use proc_macro2::Span as Span2;
use quote::quote_spanned;

pub fn comp_time_err(span: Span2, msg: &str) -> TokenStream {
    quote_spanned! {span=>
        compile_error!(#msg);
    }
    .into()
}
