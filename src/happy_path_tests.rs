use std::borrow::Cow;

use anyhow::Context;

use crate::command::Command;
use crate::command_computing::{compute_commands, parse_state_from_file_content};

const FILE_CONTENT_1: &str = r#"
FROM docker.io/library/rust:1.78.0-slim-bookworm

RUN set -eux; \
    cargo install cargo-cache --version 0.8.3 --locked; \
    # cargo install genact --version 1.4.2; \
    cargo install xh --version 0.22.0 --locked; \
    cargo cache -r all

ENV HOME="/root"
RUN mkdir -p "$HOME/.pixi/bin"
ENV PATH="$PATH:$HOME/.pixi/bin"

# Adapted from: https://github.com/prefix-dev/pixi-docker/blob/0.24.2/Dockerfile
RUN set -eux; \
    xh get --download --follow \
        "https://github.com/prefix-dev/pixi/releases/download/v0.24.2/pixi-$(uname -m)-unknown-linux-musl" \
        --output "$HOME/.pixi/bin/pixi"; \
    chmod +x "$HOME/.pixi/bin/pixi"; \
    pixi --version

WORKDIR /work
COPY pixi.toml pixi.lock /work/

RUN set -eux; \
    pixi run -e openssl-pkgconfig cargo install mdcat --version 2.1.2 --locked; \
    cargo cache -r all

RUN set -eux; \
    pixi global install git=2.45.1; \
    pixi global list

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
            "cargo install xh --version 0.22.0 --locked",
            "mkdir -p /home/denis/.pixi/bin",
            "xh get --download --follow https://github.com/prefix-dev/pixi/releases/download/v0.24.2/pixi-x86_64-unknown-linux-musl --output /home/denis/.pixi/bin/pixi",
            "chmod +x /home/denis/.pixi/bin/pixi",
            "pixi run -e openssl-pkgconfig cargo install mdcat --version 2.1.2 --locked",
            "pixi global install git=2.45.1",
        ]),
    );
}

const FILE_CONTENT_2: &str = r#"
FROM docker.io/library/rust:1.78.0-slim-bookworm

RUN set -eux; \
    cargo install cargo-cache --version 0.8.3; \
    # cargo install genact --version 1.4.2; \
    cargo install xh --version 0.22.0 --locked; \
    cargo cache -r all

ENV HOME="/root"
RUN mkdir -p "$HOME/.pixi/bin"
ENV PATH="$PATH:$HOME/.pixi/bin"

# Adapted from: https://github.com/prefix-dev/pixi-docker/blob/0.25.0/Dockerfile
RUN set -eux; \
    xh get --download --follow \
        "https://github.com/prefix-dev/pixi/releases/download/v0.25.0/pixi-$(uname -m)-unknown-linux-musl" \
        --output "$HOME/.pixi/bin/pixi"; \
    chmod +x "$HOME/.pixi/bin/pixi"; \
    pixi --version

WORKDIR /work
COPY pixi.toml pixi.lock /work/

RUN set -eux; \
    pixi run -e openssl-pkgconfig cargo install mdcat --version 2.1.2 --locked; \
    cargo cache -r all

RUN set -eux; \
    pixi global install git=2.45.2; \
    pixi global list

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
            "pixi self-update --version 0.25.0",
            "pixi global upgrade git=2.45.2",
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
            "cargo uninstall cargo-cache",
            "cargo uninstall mdcat",
            "cargo uninstall xh",
            "pixi global remove git",
            "rm /home/denis/.pixi/bin/pixi",
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
    let commands = compute_commands(&current_state, &target_state, "/home/denis", "x86_64")
        .map(Command::into_vec)
        .collect();
    Ok(commands)
}

fn split_commands<const N: usize>(commands: [&'static str; N]) -> Vec<Vec<&'static str>> {
    // I don't need `shlex::split` for my use case.
    commands.into_iter().map(|command| command.split(' ').collect::<Vec<_>>()).collect::<Vec<_>>()
}
