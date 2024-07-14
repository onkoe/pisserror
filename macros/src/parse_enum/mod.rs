use proc_macro2::Span;
use syn::{spanned::Spanned as _, DeriveInput, Ident, Item};
use variant::{WrappedVariant, WrappedVariantBuilder};

mod attr;
mod field;
mod variant;

pub struct UserEnum {
    ident: Ident,
    span: Span,
    after_span: Span,
    variants: Vec<WrappedVariant>,
}

impl UserEnum {
    /// Attempts to parse the user's given enum into its required components.
    pub fn new(input: DeriveInput) -> syn::Result<Self> {
        // check if we've been given an enum
        let (span, after_span, ident, variants) = match Item::from(input) {
            #[rustfmt::skip]
            Item::Enum(item) => {(
                    item.span(),
                    item.brace_token.span.close(),
                    item.ident,
                    item.variants // check each variant
                        .into_iter()
                        .map(|v| WrappedVariantBuilder::new(v).build())
                        .collect::<syn::Result<Vec<_>>>()?,
            )}
            other => {
                return Err(Self::err_given_non_enum(other));
            }
        };

        Ok(UserEnum {
            ident,
            span,
            after_span,
            variants,
        })
    }

    fn err_given_non_enum(item: Item) -> syn::Error {
        syn::Error::new_spanned(
            item,
            "You must use an `enum` when deriving types with `pisserror`.",
        )
    }
}
