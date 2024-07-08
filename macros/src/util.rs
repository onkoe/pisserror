//! # Util
//!
//! Some utilities to help out with writing these macros.

use proc_macro2::Span as Span2;
use syn::{punctuated::Punctuated, Ident, Path, PathSegment};

pub fn create_path(span: Span2, ident_strs: &[&str]) -> Path {
    Path {
        leading_colon: None,
        segments: {
            let mut p = Punctuated::new();
            for s in ident_strs {
                p.push(PathSegment {
                    ident: Ident::new(s, span),
                    arguments: syn::PathArguments::None,
                });
            }
            p
        },
    }
}
