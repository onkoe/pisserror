use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    punctuated::Punctuated, spanned::Spanned as _, Attribute, Ident, Meta, Path, PathSegment,
    Variant,
};

use super::{
    attr::{ErrorAttribute, FromAttribute},
    field::{self, FieldsType, WrappedField, WrappedFieldBuilder, WrappedFields},
};

/// A method to build a `WrappedVariant`.
pub struct WrappedVariantBuilder {
    variant: Variant,
}

impl WrappedVariantBuilder {
    /// Creates a new `WrappedVariantBuilder` around a given `Variant`.
    pub fn new(variant: Variant) -> Self {
        Self { variant }
    }

    /// Attempts to run all build steps to make a `WrappedVariant`.
    pub fn build(self) -> syn::Result<WrappedVariant> {
        let from_attr_checked = FromAttributeCheck::check_fields(self.variant)?;
        let error_attr_checked = ErrorAttributeCheck::check_errors(from_attr_checked)?;
        Ok(error_attr_checked.finish())
    }
}

/// Step 1: check a variant and its fields
///
/// checks that:
/// - if a variant has a `from` attr, it has no more fields.
struct FromAttributeCheck {
    /// not all variants use a `#[from]` attr
    from_attribute: Option<FromAttribute>,

    // parts of the Variant we were using
    ident: Ident,
    span: Span,
    fields: WrappedFields,
    unparsed_attrs: Vec<Attribute>,
}

impl FromAttributeCheck {
    /// Checks the given variant using the `#[from]` rules.
    pub fn check_fields(variant: Variant) -> syn::Result<Self> {
        let span = variant.span();
        let Variant {
            fields: vfields,
            attrs: vattrs,
            ident: vident,
            ..
        } = variant;

        let fields_type = match &vfields {
            syn::Fields::Named(_) => field::FieldsType::Named,
            syn::Fields::Unnamed(_) => field::FieldsType::Unnamed,
            syn::Fields::Unit => field::FieldsType::Unit,
        };

        // handles the attribute count check internally
        let fields = vfields
            .into_iter()
            .map(move |f| WrappedFieldBuilder::new(f).scan_for_froms())
            .collect::<syn::Result<Vec<_>>>()?;

        let has_from_field = fields.iter().any(move |f| f.has_from_attribute());
        let mut from = None;

        if has_from_field {
            if fields.len() > 1 {
                return Err(Self::err_nonfrom_fields_not_permitted(&span));
            }

            // nope, we're clear! let's make the thingy
            if let WrappedField::FromAttribute(from_attr_info) = fields.first().cloned().unwrap() {
                from = Some(from_attr_info);
            }
        }

        let wrapped_fields = match fields_type {
            FieldsType::Named => WrappedFields::Named(fields),
            FieldsType::Unnamed => WrappedFields::Unnamed(fields),
            FieldsType::Unit => WrappedFields::Unit,
        };

        Ok(Self {
            ident: vident,
            from_attribute: from,
            fields: wrapped_fields,
            span,
            unparsed_attrs: vattrs,
        })
    }

    /// An error asking users to remove additional fields when using
    /// the from attribute.
    pub fn err_nonfrom_fields_not_permitted(field_span: &Span) -> syn::Error {
        syn::Error::new(
            *field_span,
            "A variant containing a field with the `#[from]` attribute must only have one field. \
            Please see: https://github.com/onkoe/pisserror/issues/11#issuecomment-2215435824",
        )
    }
}

/// Step 2: check variant for an error tag
///
/// checks that:
/// - variant has an error tag.
/// - the error tag should either:
///    - have a string, or
///    - extract a string when on a `from` variant. (TODO(#15))
struct ErrorAttributeCheck {
    /// not all variants use a `#[from]` attr
    ident: Ident,
    span: Span,
    fields: WrappedFields,
    from_attribute: Option<FromAttribute>,
    error_attribute: ErrorAttribute,
}

impl ErrorAttributeCheck {
    pub fn check_errors(variant: FromAttributeCheck) -> syn::Result<Self> {
        let FromAttributeCheck {
            from_attribute,
            ident,
            span,
            fields,
            unparsed_attrs: attrs,
        } = variant;

        let error_attribute_path = crate::util::create_path(span, &["error"]);

        let mut error_attributes = attrs.iter().filter(|a| a.path() == &error_attribute_path);

        // warning: this mutates error_attributes (the iterator is being consumed)
        let (first, second) = (error_attributes.next(), error_attributes.next());
        let slice = &[first, second];

        // check if we got any problems. otherwise, grab the metalist for f-string
        let err_attr_args = match slice {
            [None, _] => {
                return Err(Self::err_missing_error_attr(span));
            }
            [Some(_), Some(second_err_attr)] => {
                return Err(Self::err_multiple_error_attrs(second_err_attr));
            }
            [Some(attr), None] => {
                let Meta::List(ref attr_args) = attr.meta else {
                    return Err(Self::err_nothing_to_display(attr));
                };

                // make sure the attribute has something inside it
                if attr_args.tokens.is_empty() {
                    return Err(Self::err_nothing_to_display(attr));
                }

                attr_args
            }
        };

        Ok(Self {
            ident,
            span,
            fields,
            from_attribute,
            error_attribute: ErrorAttribute {
                format_string: err_attr_args.tokens.clone(),
            },
        })
    }

    /// Since this is the last step, this creates the `WrappedVariant`.
    pub fn finish(self) -> WrappedVariant {
        WrappedVariant {
            ident: self.ident,
            span: self.span,
            fields: self.fields,
            from_attribute: self.from_attribute,
            error_attribute: self.error_attribute,
        }
    }

    fn err_missing_error_attr(span: Span) -> syn::Error {
        syn::Error::new(
            span,
            "Each variant must have a corresponding `#[error(...)]` attribute.",
        )
    }

    fn err_multiple_error_attrs(second_attr: &Attribute) -> syn::Error {
        syn::Error::new_spanned(
            second_attr,
            "Each variant may only have one `#[error(...)]` attribute.",
        )
    }

    fn err_nothing_to_display(attr: &Attribute) -> syn::Error {
        syn::Error::new_spanned(
            attr,
            "An `#[error(...)]` attribute must contain a format_args!() \
            f-string for implementing Display.",
        )
    }
}

pub struct WrappedVariant {
    pub ident: Ident,
    pub span: Span,
    pub fields: WrappedFields,
    pub from_attribute: Option<FromAttribute>,
    pub error_attribute: ErrorAttribute,
}
