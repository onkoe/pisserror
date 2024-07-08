use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, Ident, Variant};

use crate::util::create_path;

// pub fn create_error_impl() { // TODO?: call all these fns in here! }

/// Parses the user's enum's variants to check for any internal `#[from]`
/// attributes, then generates code that matches on any given error variant.
///
/// # Attribute Rules
///
/// The `#[from]` and `#[error]` attributes both assume they're only used once
/// per variant.
///
/// As such, the following code shouldn't compile:
///
/** ```compile_fail
use macros::Error;
use std::error::Error;

#[derive(Debug, Error)]
enum SomeError {
    // you can't have two messages!
    #[error("hi")]
    #[error("woah")]
    TwoAttrsOneField,
}
```

And for `#[from]`...

```compile_fail
use macros::Error;
use std::error::Error;

#[derive(Debug, Error)]
enum SomeError {
    // you can't have
    TwoAttrsOneField(#[from] std::io::Error, #[from] std::fmt::Error),
}
``` */
pub fn source(
    span: Span2,
    variants: &Punctuated<Variant, Comma>,
    enum_ident: Ident,
) -> syn::Result<TokenStream2> {
    let from_attr = create_path(span, &["from"]);

    // store each variant's match arm, if it's even there!
    let mut vec = vec![];

    // check for any `from` attributes on variants
    for v in variants {
        let mut t = quote! { None };
        for f in &v.fields {
            // if any of a variant's fields have the from attribute...
            if f.attrs.iter().any(|attr| *attr.meta.path() == from_attr) {
                // ...use that field in the source method impl
                let ty = f.ty.clone();
                t = quote! { Some(#ty) };
            }
        }

        let identifer = v.ident.clone();
        let fields = v.fields.clone();
        if fields.is_empty() {
            vec.push(quote! {
                #enum_ident::#identifer => #t,
            });
        } else {
            vec.push(quote! {
                #enum_ident::#identifer(#fields) => #t,
            });
        }
    }

    Ok(quote! {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match *self {
                #(#vec)*
            }
        }
    })
}

/// The method this generates is deprecated in favor of `Display`/`ToString`
/// on Error types, so we can safely return an empty string slice.
pub fn description() -> TokenStream2 {
    // TODO: consider using `Display` instead? check with other libraries b4.

    quote! {
        fn description(&self) -> &str {
            &""
        }
    }
}

/// The empty "cause" of the error. Now deprecated in favor of `source`, which
/// has the 'static bound.
///
/// As such, the method it generates always returns None.
pub fn cause() -> TokenStream2 {
    quote! {
        fn cause(&self) -> Option<&dyn Error> {
            None
        }
    }
}
