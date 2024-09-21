use std::{fmt::Display, path::Path};

use serde_json::json;

#[must_use]
pub fn quote(arg: &str) -> impl Display + '_ {
    // The Rust documentation says:
    //
    // > `Debug` implementations of types provided by the standard library (`std`, `core`, `alloc`,
    // > etc.) are not stable, and may also change with future Rust versions.
    //
    // This is why I use `format!("{}", quote(string))` instead of `format!("{string:?}")`.
    json!(arg)
}

#[must_use]
pub fn quote_path(arg: &Path) -> impl Display + '_ {
    // The Rust documentation says:
    //
    // > `Debug` implementations of types provided by the standard library (`std`, `core`, `alloc`,
    // > etc.) are not stable, and may also change with future Rust versions.
    //
    // This is why I use `format!("{}", quote(path))` instead of `format!("{path:?}")`.
    json!(arg)
}
