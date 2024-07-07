use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::quote;
use syn::{Meta, Variant};

use crate::util::create_path;

// TODO: check each variant and get info on their `#[error(...)]` attribute.

pub fn format<'a>(span: Span2, variants: impl Iterator<Item = &'a Variant>) -> TokenStream2 {
    // for v in variants
    //      for attr in v.attrs
    //          if attr is #[error(...)]
    //              add to output
    //          else
    //              compile_error!("fuck you")

    // attribute that looks like `#[error(...)]`.
    let error_attr = create_path(span, &["error"]);

    // output
    let mut vec = vec![];

    // make sure each variant has the error attr. grab each one for Display
    for v in variants {
        for attr in &v.attrs {
            if attr.meta.path() == &error_attr {
                // TODO: maybe respect inherited Display on `#[from]` variants
                //       where we get Meta::Path instead.

                // complain if user gave didn't give anything to
                let Meta::List(ref attr_args) = attr.meta else {
                    return quote! {
                        compile_error!(
                            "All variants must be given something to print, as\
                            the trait is defined as: `Error: Debug + Display`.");
                    };
                };

                // TODO: parse attr args correctly!!!
                vec.push(quote! {
                    #v.ident => {format!(#attr_args)}
                });
            } else {
                return quote! {
                    compile_error!("Each variant must have a corresponding `#[error(...)` attribute.");
                };
            }
        }
    }

    todo!()
}
