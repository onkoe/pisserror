use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, Ident, Meta, Variant};

use crate::util::{create_path, variant::make_match_head};

/**
To implement `Display`, we need to parse the given error message for each
variant.

However, there needs to be one error attribute per - not more, not less.

I've made some tests below verifying this assumption.

First, having no `#[error]` attribute should fail:
```compile_fail
use macros::Error;
use std::error::Error;

#[derive(Debug, Error)]
#[allow(unused)]
enum MyError {
    VariantOne,
}
```

Also, you can't have too many of them, either:

```compile_fail
use macros::Error;
use std::error::Error;

#[derive(Debug, Error)]
#[allow(unused)]
enum MyError {
    #[error("first attr")]
    #[error("second attr")]
    VariantOne,
}
``` */
pub fn fmt(
    span: Span2,
    variants: &Punctuated<Variant, Comma>,
    enum_name: &Ident,
) -> syn::Result<TokenStream2> {
    // just an attribute that looks like `#[error(...)]`.
    let error_attr = create_path(span, &["error"]);

    // all the lines of `match`
    let mut vec = vec![];

    // make sure each variant has the error attribute.
    // then, grab each one for use in the impl
    for v in variants {
        let mut has_error_attribute = false;

        for attr in &v.attrs {
            if attr.meta.path() == &error_attr {
                // TODO: maybe respect inherited Display on `#[from]` variants
                //       where we get Meta::Path instead.

                // complain if user gave didn't give an error message
                let Meta::List(ref attr_args) = attr.meta else {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "All variants must be given \
                        something to print, as the trait is defined as: `Error: Debug + Display`.",
                    ));
                };

                // complain if user used multiple error attrs on one variant
                if has_error_attribute {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "Each variant may only have one `#[error(...)]` attribute.",
                    ));
                }

                // make sure the attribute has something inside of it
                // TODO: when a `#[from]` attr is present, don't check for this.
                if attr_args.tokens.is_empty() {
                    return Err(syn::Error::new_spanned(
                        attr_args,
                        "An `#[error(...)]` attribute must contain a value.",
                    ));
                }

                // TODO: parse attr args correctly!!!
                has_error_attribute = true;
                let match_head = make_match_head(enum_name, v);
                let tokens = &attr_args.tokens;
                vec.push(quote! {
                    #match_head => {f.write_str(format!(#tokens).as_str())},
                });
            }
        }

        // if we don't have an error attribute, complain
        if !has_error_attribute {
            return Err(syn::Error::new_spanned(
                v,
                "Each variant must have a corresponding `#[error(...)]` attribute.",
            ));
        }
    }

    Ok(quote! {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
            match *self {
                #(#vec)*
            }
        }
    })
}
