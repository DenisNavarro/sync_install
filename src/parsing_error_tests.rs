use std::fmt;

use anyhow::{Context as _, ensure};

use crate::command_computing::parse_state_from_file_content;

#[test]
fn cargo_install_without_expected_suffix() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        "RUN cargo install fsays --version 0.3.0 --locked",
        [
            "failed to parse line 1: ",
            r#"line with "cargo install " but which does not end with "; \""#,
        ],
    )
}

#[test]
fn cargo_install_without_crate_name() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r"RUN set -eux; \
            cargo install ; \
            cargo cache -r all",
        ["failed to parse line 2: ", "empty crate name"],
    )
}

#[test]
fn same_crate_in_a_previous_line() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r"RUN set -eux; \
            cargo install fsays --version 0.1.0 --locked; \
            cargo install fsays --version 0.3.0 --locked; \
            cargo cache -r all",
        [
            "failed to parse line 3: ",
            r#""fsays" crate already installed in a previous line: "#,
            "the command was [cargo install fsays --version 0.1.0 --locked]",
        ],
    )
}

#[test]
fn pixi_global_install_without_expected_suffix() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        "RUN pixi global install git=2.48.1",
        [
            "failed to parse line 1: ",
            r#"line with "pixi global install " but which does not end with "; \""#,
        ],
    )
}

#[test]
fn pixi_global_install_without_expected_prefix() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r"RUN set -eux; \
            echo pixi global install git=2.48.1; \
            pixi global list",
        [
            "failed to parse line 2: ",
            r#"left trimmed line with "pixi global install " but which "#,
            r#"does not start with "pixi global install ""#,
        ],
    )
}

#[test]
fn pixi_global_install_without_recipe_and_version() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r"RUN set -eux; \
            pixi global install ; \
            pixi global list",
        ["failed to parse line 2: ", "neither recipe nor version"],
    )
}

#[test]
fn pixi_global_install_without_equal() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r"RUN set -eux; \
            pixi global install git; \
            pixi global list",
        ["failed to parse line 2: ", "'=' is missing"],
    )
}

#[test]
fn pixi_global_install_with_empty_recipe() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r"RUN set -eux; \
            pixi global install =2.48.1; \
            pixi global list",
        ["failed to parse line 2: ", "empty recipe"],
    )
}

#[test]
fn same_recipe_in_a_previous_line() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r"RUN set -eux; \
            pixi global install git=2.46.0; \
            pixi global install git=2.48.1; \
            pixi global list",
        [
            "failed to parse line 3: ",
            r#""git" recipe already installed in a previous line: it was git=2.46.0"#,
        ],
    )
}

fn parse_first_arg_and_check_error_contains<const N: usize>(
    file_content: &'static str,
    texts: [&'static str; N],
) -> anyhow::Result<()> {
    let result = parse_state_from_file_content(file_content);
    check_err_contains(result, texts)
}

fn check_err_contains<T, E>(
    result: Result<T, E>,
    texts: impl IntoIterator<Item = impl AsRef<str>>,
) -> anyhow::Result<()>
where
    E: fmt::Debug,
{
    let error = result.err().context("missing error")?;
    let msg = format!("{error:?}");
    for text in texts {
        let text = text.as_ref();
        ensure!(msg.contains(text), "the error message {msg:?} does not contain {text:?}");
    }
    Ok(())
}
