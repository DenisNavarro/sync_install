use std::borrow::Cow;

use anyhow::Context as _;

use crate::command::Command;
use crate::command_computing::{compute_commands, parse_state_from_file_content};

const FILE_CONTENT_1: &str = r#"
FROM docker.io/library/rust:1.81.0-slim-bookworm

RUN set -eux; \
    cargo install cargo-cache --version 0.8.3 --locked; \
    # cargo install fsays --version 0.3.0 --locked; \
    cargo install pixi --git https://github.com/prefix-dev/pixi.git --tag v0.35.0 --locked; \
    cargo cache -r all

WORKDIR /work
COPY pixi.toml pixi.lock /work/

RUN set -eux; \
    pixi run -e openssl-pkgconfig cargo install cargo-update --version 14.1.1 --locked; \
    cargo cache -r all

RUN set -eux; \
    pixi global install git=2.45.2; \
    pixi global list

ENV HOME="/root"
ENV PATH="$PATH:$HOME/.pixi/bin"

CMD ["/bin/bash"]
"#;

#[test]
fn install() {
    let current_state_file_content = "";
    let target_state_file_content = FILE_CONTENT_1;
    assert_eq!(
        parse_args_and_compute_commands(current_state_file_content, target_state_file_content)
            .unwrap(),
        split_commands([
            "cargo install cargo-cache --version 0.8.3 --locked",
            "cargo install pixi --git https://github.com/prefix-dev/pixi.git --tag v0.35.0 --locked",
            "pixi run -e openssl-pkgconfig cargo install cargo-update --version 14.1.1 --locked",
            "pixi global install git=2.45.2",
        ]),
    );
}

const FILE_CONTENT_2: &str = r#"
FROM docker.io/library/rust:1.81.0-slim-bookworm

RUN set -eux; \
    cargo install cargo-cache --version 0.8.3; \
    # cargo install fsays --version 0.3.0 --locked; \
    cargo install pixi --git https://github.com/prefix-dev/pixi.git --tag v0.35.0 --locked; \
    cargo cache -r all

WORKDIR /work
COPY pixi.toml pixi.lock /work/

RUN set -eux; \
    pixi run -e openssl-pkgconfig cargo install cargo-update --version 14.1.1 --locked; \
    cargo cache -r all

RUN set -eux; \
    pixi global install git=2.46.0; \
    pixi global list

ENV HOME="/root"
ENV PATH="$PATH:$HOME/.pixi/bin"

CMD ["/bin/bash"]
"#;

#[test]
fn update() {
    let current_state_file_content = FILE_CONTENT_1;
    let target_state_file_content = FILE_CONTENT_2;
    assert_eq!(
        parse_args_and_compute_commands(current_state_file_content, target_state_file_content)
            .unwrap(),
        split_commands([
            "cargo install cargo-cache --version 0.8.3 --force",
            "pixi global install git=2.46.0",
        ]),
    );
}

#[test]
fn remove() {
    let current_state_file_content = FILE_CONTENT_2;
    let target_state_file_content = "";
    assert_eq!(
        parse_args_and_compute_commands(current_state_file_content, target_state_file_content)
            .unwrap(),
        split_commands([
            "pixi global uninstall git",
            "cargo uninstall cargo-update",
            "cargo uninstall pixi",
            "cargo uninstall cargo-cache",
        ]),
    );
}

#[test]
fn no_change() {
    assert_eq!(
        parse_args_and_compute_commands(FILE_CONTENT_1, FILE_CONTENT_1).unwrap(),
        Vec::<Vec<Cow<'static, str>>>::new()
    );
    assert_eq!(
        parse_args_and_compute_commands(FILE_CONTENT_2, FILE_CONTENT_2).unwrap(),
        Vec::<Vec<Cow<'static, str>>>::new()
    );
}

fn parse_args_and_compute_commands(
    current_state_file_content: &'static str,
    target_state_file_content: &'static str,
) -> anyhow::Result<Vec<Vec<Cow<'static, str>>>> {
    let current_state = parse_state_from_file_content(current_state_file_content)
        .context("failed to parse the current state file content")?;
    let target_state = parse_state_from_file_content(target_state_file_content)
        .context("failed to parse the target state file content")?;
    Ok(compute_commands(&current_state, &target_state).map(Command::into_vec).collect())
}

fn split_commands<const N: usize>(commands: [&'static str; N]) -> Vec<Vec<&'static str>> {
    // I don't need `shlex::split` for my use case.
    commands.into_iter().map(|command| command.split(' ').collect::<Vec<_>>()).collect::<Vec<_>>()
}
