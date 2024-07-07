//! # Derive
//!
//! Contains the `#[derive(Error)]` part of pisserror.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned as _, DeriveInput, Item, Variant};

pub fn derive_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input_span = input.span();

    // make sure we've been given an enum.
    if let Item::Enum(item) = Item::from(input) {
        let name = item.ident;
        let variants = item.variants;

        // Rust's Error type is defined as: `Error: Debug + Display`. to satisfy
        // `Debug`, we need to make sure the enum implements it.
        //
        // let debug_check = ...; // TODO: do some fancy checks. maybe assertions with an internal type?
        //
        // HEY! the compiler already does this for us! a nice error message might be preferable, though!

        // assemble the match arms
        let source = source(after_span, variants.iter()); // TODO: check after_span
        let description = description();
        let cause = cause();

        // put all those together!
        let impl_block = quote_spanned! {input_span=>
            // TODO: check each variant and get info on their `#[error(...)]` attribute.
            #[automatically_derived]
            impl std::error::Error for #name {
                #source
                #description
                #cause

                compile_error!("TODO: Error is not yet implemented.")
            }

            #[automatically_derived]
            impl core::fmt::Display for #name {
                compile_error!("TODO: Display is not yet implemented.")
            }
        };

        impl_block.into()
    } else {
        // remind user to use an enum.
        quote_spanned! {input_span=>
            compile_error!("You must use an `enum` when deriving types with `pisserror`.");
        }
        .into()
    }
}

/// Parses the user's enum's variants to check for any internal `#[from]`
/// attributes, then generates code that matches on any given error variant.
fn source<'a>(span: Span, variants: impl Iterator<Item = &'a Variant>) -> TokenStream2 {
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
fn description() -> TokenStream2 {
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
fn cause() -> TokenStream2 {
    quote! {
        fn cause(&self) -> Option<&dyn Error> {
            None
        }
    }
}
