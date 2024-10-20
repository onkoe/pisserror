/*!# pisserror

A golden replacement for `thiserror`.

## Usage

You'll likely find `pisserror` to be pretty familiar. As with `thiserror`, it derives `Error` for any enum you give to it. Here's a sample of its current usage:

```
use pisserror::Error;
use std::error::Error; // we don't mess with your prelude :D

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("encountered a disk error. see: {_0}")]
    DiskError(#[from] std::io::Error),
    #[error("key `{_0}` has no matching data records")]
    KeyNotFound(String),
    #[error("attempted to store a malformed header. expected: `{expected:?}`. got: `{got:?}`")]
    MalformedHeader {
        expected: String,
        got: String,
    },
    #[error("other error: {_0}")]
    Other(String),
}
```

Also, you may wish to note that `pisserror` works with `#![no_std]`/embedded projects! Just ask `cargo add` to not use default features, like `cargo add pisserror --no-default-features`.

Alternatively, you can add it to `Cargo.toml` by adding `default-features = false`:

```toml
[dependencies]
pisserror = { version = (your version), default-features = false }
```

## Feature Requests and Problems

If there's something wrong or missing, please create a GitHub issue! Make sure to thoroughly describe your intentions.

## Contributions

Contributions are welcome! Please create a PR and explain the changes you made.

There are a few ground rules, though:

- All contributions are bound by the [license](./LICENSE).
- Commits must be in [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) format. Feel free to interpret the rules a bit, though!
- Document and test your work.

### Making a Release

There are only a few steps to releasing this crate.

1. Generate the `README.md` file using `cargo-rdme`.
1. Change the version numbers in `/Cargo.toml` and `/macros/Cargo.toml`.
    - Under the `dependencies.pisserror_macros` section, you must also change the version number to match the new release.
1. Run `cargo publish`!
*/

#[cfg(doctest)]
pub mod _doctests;

pub use pisserror_macros::Error;
