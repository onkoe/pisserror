use proc_macro2::Span;
use syn::{spanned::Spanned as _, DeriveInput, Generics, Ident, Item};
use variant::{WrappedVariant, WrappedVariantBuilder};

pub(crate) mod attr;
pub(super) mod field;
pub(super) mod variant;

pub(crate) struct UserEnum {
    ident: Ident,
    generics: Generics,
    span: Span,
    after_span: Span,
    variants: Vec<WrappedVariant>,
}

impl UserEnum {
    /// Attempts to parse the user's given enum into its required components.
    pub(crate) fn new(input: DeriveInput) -> syn::Result<Self> {
        // check if we've been given an enum
        let (span, generics, after_span, ident, variants) = match Item::from(input) {
            #[rustfmt::skip]
            Item::Enum(item) => {(
                    item.span(),
                    item.generics,
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

        Ok(Self {
            ident,
            generics,
            span,
            after_span,
            variants,
        })
    }

    /// The given enum's identifier (name).
    pub(crate) fn ident(&self) -> Ident {
        self.ident.clone()
    }

    /// Generic bounds (including lifetimes) on the enum.
    pub(crate) fn generics(&self) -> &Generics {
        &self.generics
    }

    /// The source region of the given enum.
    pub(crate) const fn span(&self) -> Span {
        self.span
    }

    /// A span right outside of the enum's definition.
    pub(crate) const fn after_span(&self) -> Span {
        self.after_span
    }

    /// The available variants on the given enum.
    pub(crate) const fn variants(&self) -> &Vec<WrappedVariant> {
        &self.variants
    }

    fn err_given_non_enum(item: Item) -> syn::Error {
        syn::Error::new_spanned(
            item,
            "You must use an `enum` when deriving types with `pisserror`.",
        )
    }
}
