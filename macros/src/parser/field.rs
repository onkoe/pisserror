use proc_macro2::Span;
use syn::{spanned::Spanned as _, Attribute, Field, Ident, Type};

use crate::util;

/// Something like `syn::Fields`, but all the fields have been checked.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum WrappedFields {
    Named(Vec<WrappedField>),
    Unnamed(Vec<WrappedField>),
    Unit,
}

/// `syn::Fields`, minus the fields. This is here to avoid some nasty logic.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub(crate) enum FieldsType {
    Named,
    Unnamed,
    Unit,
}

/// Some information about a field.
#[derive(Clone, Debug)]
pub(crate) struct WrappedFieldInfo {
    pub ident: Option<Ident>,
    pub ty: Type,
    pub span: Span,
}

impl PartialEq for WrappedFieldInfo {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident && self.ty == other.ty
    }
}

/// A field.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum WrappedField {
    Typical(WrappedFieldInfo),
    FromAttribute(WrappedFieldInfo),
}

impl WrappedField {
    /// Checks if this field has the `#[from]` attribute.
    pub(crate) const fn has_from_attribute(&self) -> bool {
        match *self {
            Self::FromAttribute(_) => true,
            Self::Typical(_) => false,
        }
    }
}

pub(crate) struct WrappedFieldBuilder {
    field: Field,
}

impl WrappedFieldBuilder {
    pub(crate) const fn new(field: Field) -> Self {
        Self { field }
    }

    /// Runs all build steps to create a `WrappedField`.
    pub(crate) fn build(self) -> syn::Result<WrappedField> {
        let split = FromAttributeSplit::split_field(self.field);
        let checked = FromAttributeCheck::check_from(split)?;
        Ok(checked.finish())
    }
}

pub(crate) struct FromAttributeSplit {
    field_info: WrappedFieldInfo,
    attributes: Vec<Attribute>,
}

impl FromAttributeSplit {
    /// Takes the info and attributes from a given field, consuming the field.
    pub(crate) fn split_field(field: Field) -> Self {
        let span = field.span();

        let Field {
            ident, ty, attrs, ..
        } = field;

        Self {
            field_info: WrappedFieldInfo { ident, ty, span },
            attributes: attrs,
        }
    }
}

pub(crate) struct FromAttributeCheck {
    wrapped_field: WrappedField,
}

impl FromAttributeCheck {
    /// Checks this field for a `from` attribute. Returns an error if the
    /// `from` rules are violated.
    pub(crate) fn check_from(split: FromAttributeSplit) -> syn::Result<Self> {
        let (field_info, attrs) = (split.field_info, split.attributes);
        let field_span = &field_info.span;
        let from_attribute_path = &util::create_path(field_info.span, &["from"]);

        let mut already_found_from_attribute = false;

        for attr in attrs {
            if attr.path() == from_attribute_path {
                if already_found_from_attribute {
                    return Err(Self::err_too_many_from_attributes(*field_span));
                }

                // check if the attr has some args
                match attr.meta {
                    syn::Meta::List(_) | syn::Meta::NameValue(_) => {
                        return Err(Self::err_from_attribute_has_args(attr.span()))
                    }
                    syn::Meta::Path(_) => (), // good. there are no args in `#[from]`
                }

                // if there are lifetimes on the from type, complain!
                if let Type::Reference(ref inner) = field_info.ty {
                    // but only if it's not static
                    if let Some(ref lt) = inner.lifetime {
                        if lt.ident != "static" {
                            return Err(Self::err_from_attribute_field_has_nonstatic_lifetime(
                                *field_span,
                                lt.ident.span(),
                            ));
                        }
                    }
                }

                already_found_from_attribute = true;
            }
        }

        Ok(Self {
            wrapped_field: if already_found_from_attribute {
                WrappedField::FromAttribute(field_info)
            } else {
                WrappedField::Typical(field_info)
            },
        })
    }

    pub(crate) fn err_too_many_from_attributes(field_span: Span) -> syn::Error {
        syn::Error::new(
            field_span,
            "You may only have one `#[from]` attribute per variant.",
        )
    }

    pub(crate) fn err_from_attribute_has_args(attribute_span: Span) -> syn::Error {
        syn::Error::new(
            attribute_span,
            "The `#[from]` attribute does not take any arguments, but some were found.",
        )
    }

    fn err_from_attribute_field_has_nonstatic_lifetime(
        field_span: Span,
        lt_span: Span,
    ) -> syn::Error {
        let mut err = syn::Error::new(
            field_span,
            "Cannot implement `Error::source`. This type must be owned or use a 'static reference. ",
        );

        err.combine(syn::Error::new(
            lt_span,
            "Try changing this lifetime to be 'static, or remove it altogether.",
        ));

        err
    }

    /// Returns the created WrappedField, consuming `self`.
    pub(crate) fn finish(self) -> WrappedField {
        self.wrapped_field
    }
}
