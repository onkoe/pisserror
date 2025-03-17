//! Ensure that the parser respects type lifetimes.
//!
//! Just in case anyone wants to suffer by including them in their error types.

use core::error::Error;
use pisserror_macros::Error;

/// An error we'll use in other errors (to test `#[from]`).
#[derive(Clone, Debug, Error)]
enum SomeOtherError {
    #[error("variant")]
    Variant,
}

/// Ensure lifetimes can be used inside of error types.
#[test]
fn lifetimes() {
    #[derive(Clone, Debug, Error)]
    enum ErrorWithLifetimes<'a> {
        #[error("static lt")]
        LtStatic(&'static str),

        #[error("non-static lt")]
        LtTickA(&'a str),

        #[error("from variant")]
        FromOtherError(#[from] SomeOtherError),
    }

    let err_msg = String::from("hello world");

    // scope will lower lifetime
    {
        let err_msg_ref = err_msg.as_str();
        let e = ErrorWithLifetimes::LtTickA(err_msg_ref);

        // `Display` works
        println!("yay! an error: {e}");
    }

    // `From` works
    {
        let other_err = SomeOtherError::Variant;
        let _err_w_lts = ErrorWithLifetimes::from(other_err);
    }

    // `Debug` works
    {
        let static_variant = ErrorWithLifetimes::LtStatic("im static lol");
        println!("{static_variant:?}");
    }

    //
}

/// Check that the `From` implementation works with static lifetimes.
///
/// Note: this is insane - you probably shouldn't do it.
#[test]
fn from_with_static_lifetime() {
    /// Another error that we'll borrow.
    #[derive(Clone, Debug, Error)]
    enum ErrorTyWeWillBorrowTickStatic {
        #[error("variant")]
        Variant,
    }

    /// An error containing a static reference to another error.
    ///
    /// You uh... shouldn't do this. But it's supported nonetheless.
    #[derive(Clone, Debug, Error)]
    enum MyError {
        #[error("from err with static lifetime")]
        FromStaticLt(#[from] &'static ErrorTyWeWillBorrowTickStatic),
    }

    // actually do the borrowing lol
    const WILL_BORROW: ErrorTyWeWillBorrowTickStatic = ErrorTyWeWillBorrowTickStatic::Variant;
    let _my_error = MyError::from(&WILL_BORROW); // `From` works
}

/// Checks if a combo of lifetimes and generics works alright.
mod _lifetimes_and_generics {
    use core::error::Error;
    use pisserror_macros::Error;
    use std::borrow::Cow;

    use super::SomeOtherError;

    #[derive(Debug, Error)]
    #[expect(unused)]
    #[expect(variant_size_differences)]
    enum MyError<'a, 'b, T: core::fmt::Debug, Q: core::fmt::Debug + 'b> {
        #[error("borrowed generic")]
        RefGeneric { t: &'a T, q: Q },

        #[error("cow str")]
        CowStr(Cow<'b, str>),

        #[error("from")]
        From(#[from] SomeOtherError),

        #[error("frankly, too many borrows")]
        LottaBorrows {
            f1: &'a mut &'b mut [u8],
            f2: &'a &'b mut T,
            f3: Box<[u8]>,
            f4: *const [u8],
            f5: &'b mut str,
            f6: &'a &'b [u8],
            f7: &'a MyError<'a, 'b, T, Q>,
        },
    }

    #[test]
    fn lifetimes_and_generics() {
        let err: MyError<'_, '_, String, &str> = MyError::CowStr(Cow::Borrowed("hi"));
        println!("{err}"); // display
        println!("{err:?}"); // debug

        // from
        let _err_from: MyError<'_, '_, &[u8], &[u8]> = MyError::from(SomeOtherError::Variant);
    }
}
