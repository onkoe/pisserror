use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::Variant;

use crate::util::create_path;

/// pub fn create_error_impl() { // TODO?: call all these fns in here! }

/// Parses the user's enum's variants to check for any internal `#[from]`
/// attributes, then generates code that matches on any given error variant.
pub fn source<'a>(span: Span, variants: impl Iterator<Item = &'a Variant>) -> TokenStream2 {
    let from_attr = create_path(span, &["from"]);

    // make a new hashmap to store variants' attribute field, if it's even there!
    let mut vec = vec![];

    // check for any `from` attributes on variants
    for v in variants {
        let mut t = None;
        for f in &v.fields {
            // if any of a variant's fields have the from attribute...
            if f.attrs.iter().any(|attr| *attr.meta.path() == from_attr) {
                // ...use that field in the source method impl
                t = Some(f.ty.clone());
            }
        }

        let identifer = v.ident.clone();
        vec.push(quote! {
            #identifer(#v.fields*) => #t,
        });
    }

    // TODO: use the vec
    quote! {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            None
        }
    }
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
