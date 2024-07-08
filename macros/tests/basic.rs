use macros::Error;
use std::error::Error;

#[derive(Debug, Error)]
#[allow(unused)]
enum MyErrorType {
    #[error("1 {}", "am i a genius?")]
    Thing1,
    #[error("2")]
    Thing2,
}

#[test]
fn farts() -> Result<(), MyErrorType> {
    let error = MyErrorType::Thing1;
    eprintln!("{}", error);
    Ok(())
}
