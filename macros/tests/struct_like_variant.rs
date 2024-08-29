use pisserror_macros::Error;
use std::error::Error;

// An error type whose one variant is struct-like
#[derive(Debug, Error)]
#[allow(unused)]
enum MyError {
    #[error("hey what's up")]
    InformativeError { expected: i32, got: i32 },
}
