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
        r"RUN set -eux; \
            pixi global install git=2.49.0",
        [
            "failed to parse line 2: ",
            r#"line with "pixi global install " but which does not end with "; \""#,
        ],
    )
}

#[test]
fn pixi_global_install_without_recipe_and_version() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r"RUN set -eux; \
            pixi global install ; \
            pixi clean cache --yes",
        ["failed to parse line 2: ", "neither recipe nor version"],
    )
}

#[test]
fn pixi_global_install_without_equal() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r"RUN set -eux; \
            pixi global install git; \
            pixi clean cache --yes",
        ["failed to parse line 2: ", "'=' is missing"],
    )
}

#[test]
fn pixi_global_install_with_empty_recipe() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r"RUN set -eux; \
            pixi global install =2.49.0; \
            pixi clean cache --yes",
        ["failed to parse line 2: ", "empty recipe"],
    )
}

#[test]
fn same_recipe_in_a_previous_line() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r"RUN set -eux; \
            pixi global install git=2.46.0; \
            pixi global install git=2.49.0; \
            pixi clean cache --yes",
        [
            "failed to parse line 3: ",
            r#""git" recipe already installed in a previous line: it was git=2.46.0"#,
        ],
    )
}

#[test]
fn git_config_set_global_without_expected_suffix() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r"RUN set -eux; \
            git config set --global init.defaultBranch main",
        [
            "failed to parse line 2: ",
            r#"line with "git config set --global " but which does not end with "; \""#,
        ],
    )
}

#[test]
fn git_config_set_global_without_value() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r#"RUN set -eux; \
            git config set --global init.defaultBranch; \
            cat "$HOME/.gitconfig""#,
        ["failed to parse line 2: ", r#""init.defaultBranch" git global option without value"#],
    )
}

#[test]
fn git_config_set_global_with_empty_option() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r#"RUN set -eux; \
            git config set --global  main; \
            cat "$HOME/.gitconfig""#,
        ["failed to parse line 2: ", "empty option"],
    )
}

#[test]
fn git_config_set_global_with_empty_value_1() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r#"RUN set -eux; \
            git config set --global init.defaultBranch ; \
            cat "$HOME/.gitconfig""#,
        ["failed to parse line 2: ", "empty value"],
    )
}

#[test]
fn git_config_set_global_with_empty_value_2() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r#"RUN set -eux; \
            git config set --global init.defaultBranch ''; \
            cat "$HOME/.gitconfig""#,
        ["failed to parse line 2: ", "empty value"],
    )
}

#[test]
fn git_config_set_global_without_ending_apostrophe() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r#"RUN set -eux; \
            git config set --global user.name 'John Smith; \
            cat "$HOME/.gitconfig""#,
        ["failed to parse line 2: ", r#"missing ending apostrophe in "'John Smith""#],
    )
}

#[test]
fn same_git_global_option_in_a_previous_line() -> anyhow::Result<()> {
    parse_first_arg_and_check_error_contains(
        r#"RUN set -eux; \
            git config set --global init.defaultBranch master; \
            git config set --global init.defaultBranch main; \
            cat "$HOME/.gitconfig""#,
        [
            "failed to parse line 3: ",
            r#""init.defaultBranch" git global option already set in a previous line: "#,
            r#"the value was "master""#,
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
