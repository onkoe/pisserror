//! This tests for error variants that use its fields within the error message.

use pisserror_macros::Error;
use std::error::Error;

/// An error type that has no variants (completely valid)
#[derive(Debug, Error)]
#[allow(unused)]
enum FieldFmtdError {
    #[error("my name is: {}", &name)]
    MyStructlikeVariant { name: String },
    #[error("my favorite color is: {_0}")] // FIXME: this is cruel
    MyTuplelikeVariant(String),
}

#[test]
fn construct_fmtd_err() {
    let structlike_err = FieldFmtdError::MyStructlikeVariant {
        name: String::from("piss"),
    };
    let tuplelike_err = FieldFmtdError::MyTuplelikeVariant(String::from("yellow"));

    assert_eq!("my name is: piss", structlike_err.to_string());
    assert_eq!("my favorite color is: yellow", tuplelike_err.to_string());
}
