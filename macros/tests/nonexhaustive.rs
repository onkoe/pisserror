use pisserror_macros::Error;
use std::error::Error;

/// An error type that has no variants (completely valid)
#[derive(Debug, Error)]
#[allow(unused)]
#[non_exhaustive]
enum NonexhaustiveError {
    #[error("hi")]
    Variant,
}

#[test]
fn _nonexhaustive() {}
