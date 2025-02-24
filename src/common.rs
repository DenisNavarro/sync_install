use std::{fmt::Display, path::Path};

use uniquote::Quote as _;

#[must_use]
pub fn quote(string: &str) -> impl Display {
    // The Rust documentation says:
    //
    // > `Debug` implementations of types provided by the standard library (`std`, `core`, `alloc`,
    // > etc.) are not stable, and may also change with future Rust versions.
    //
    // This is why I use `format!("{}", quote(string))` instead of `format!("{string:?}")`.
    string.quote()
}

#[must_use]
pub fn quote_path(path: &Path) -> impl Display {
    // The Rust documentation says:
    //
    // > `Debug` implementations of types provided by the standard library (`std`, `core`, `alloc`,
    // > etc.) are not stable, and may also change with future Rust versions.
    //
    // It also says that `std::path::Path::display` "may perform lossy conversion".
    //
    // This is why I use `format!("{}", quote_path(path))` instead of `format!("{path:?}")` or
    // `format!("{}", path.display())`.
    path.quote()
}
