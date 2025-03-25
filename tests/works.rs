//! Checks that the macro is re-exported in the base `pisserror` crate.
//!
//! Note that all other tests are in `macros/tests/` instead!

use core::error::Error;
use pisserror::Error;

/// Some error type. It's mine.
#[derive(Clone, Debug, PartialEq, PartialOrd, Hash, Error)]
pub enum MyError {
    #[error("hello {_0}")]
    Hello(String),
}

#[test]
fn _is_reexported() {
    let hello_world = MyError::Hello("world".into());
    assert_eq!(hello_world.to_string(), "hello world");
}
