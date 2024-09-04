use core::error::Error;
use pisserror_macros::Error;

#[derive(Debug, Error)]
#[expect(unused, reason = "just a compile test")]
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
    #[expect(
        clippy::allow_attributes,
        reason = "testing regular external attribute handling"
    )]
    #[allow(non_snake_case)]
    #[allow(non_snake_case)]
    Thing1,
    /// I can even document the variants
    #[error("2")]
    Thing2,
}
