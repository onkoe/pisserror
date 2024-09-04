use core::error::Error;
use pisserror_macros::Error;

// An error type whose one variant is struct-like
#[derive(Debug, Error)]
#[expect(unused, reason = "just a compile test")]
enum MyError {
    #[error("hey what's up")]
    InformativeError { expected: i32, got: i32 },
}
