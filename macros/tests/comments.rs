use pisserror_macros::Error;
use std::error::Error;

#[derive(Debug, Error)]
#[allow(unused)]
enum MyError {
    // 1. hey hows it going
    // 2. yo pretty good how r u
    // 1. tired
    // 2. aw
    //
    #[error("1 {}", "am i a genius?")]
    // ooo this is a weird place for a comment
    // and yet...
    // it works
    #[allow(clippy::allow_attributes)]
    #[allow(non_snake_case)]
    #[allow(non_snake_case)]
    Thing1,
    /// I can even document the variants
    #[error("2")]
    Thing2,
}
