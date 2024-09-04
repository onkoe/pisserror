use proc_macro2::TokenStream;

use super::field::WrappedFieldInfo;

// #[from] contains a name and type. but that's just a field!
pub(crate) type FromAttribute = WrappedFieldInfo;

/// An attribute that describes a specific error variant.
///
/// Can either look like `#[error("some message here")]` or `#[error(transparent)]`
/// for variants with the `#[from]` attribute.
#[derive(Debug)]
pub(crate) enum ErrorAttribute {
    Stringy(TokenStream),
    Transparent,
}

impl ErrorAttribute {
    pub(crate) const TRANSPARENT_LITERAL: &'static str = "transparent";
}
