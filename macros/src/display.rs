use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::quote;
use syn::{Ident, Meta, Variant};

use crate::util::create_path;

// TODO: check each variant and get info on their `#[error(...)]` attribute.

pub fn fmt<'a>(
    span: Span2,
    variants: impl Iterator<Item = &'a Variant>,
    enum_ident: Ident,
) -> syn::Result<TokenStream2> {
    // just an attribute that looks like `#[error(...)]`.
    let error_attr = create_path(span, &["error"]);

    // all the lines of `match`
    let mut vec = vec![];

    // make sure each variant has the error attribute.
    // then, grab each one for use in the impl
    for v in variants {
        for attr in &v.attrs {
            if attr.meta.path() == &error_attr {
                // TODO: maybe respect inherited Display on `#[from]` variants
                //       where we get Meta::Path instead.

                // complain if user gave didn't give an error message
                let Meta::List(ref attr_args) = attr.meta else {
                    let err_msg = "All variants must be given something to print, as\
                        the trait is defined as: `Error: Debug + Display`.";
                    return Err(syn::Error::new_spanned(attr, err_msg));
                };

                // TODO: parse attr args correctly!!!
                let variant_ident = &v.ident;
                let tokens = &attr_args.tokens;
                vec.push(quote! {
                    &#enum_ident::#variant_ident => {f.write_str(format!(#tokens).as_str())},
                });
            } else {
                return Err(syn::Error::new_spanned(
                    v,
                    "Each variant must have a corresponding `#[error(...)` attribute.",
                ));
            }
        }
    }

    Ok(quote! {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
            match self {
                #(#vec)*
            }
        }
    })
}
