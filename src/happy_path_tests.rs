use anyhow::Context as _;

use crate::command::Command;
use crate::command_computing::{compute_commands, parse_state_from_file_content};

const FILE_CONTENT_1: &str = include_str!("../dockerfiles/tested_example_1");
const FILE_CONTENT_2: &str = include_str!("../dockerfiles/tested_example_2");

#[test]
fn install() {
    let current_state_file_content = "";
    let target_state_file_content = FILE_CONTENT_1;
    assert_eq!(
        parse_args_and_compute_commands(current_state_file_content, target_state_file_content)
            .unwrap(),
        split_commands([
            "cargo install cargo-cache --version 0.8.3 --locked",
            "cargo install pixi --git https://github.com/prefix-dev/pixi.git --tag v0.41.4 --locked",
            "pixi run -e openssl-pkgconfig cargo install cargo-update --version 16.1.0 --locked",
            "pixi global install git=2.46.0",
        ]),
    );
}

#[test]
fn update() {
    let current_state_file_content = FILE_CONTENT_1;
    let target_state_file_content = FILE_CONTENT_2;
    assert_eq!(
        parse_args_and_compute_commands(current_state_file_content, target_state_file_content)
            .unwrap(),
        split_commands([
            "cargo install cargo-cache --version 0.8.3 --force",
            "pixi global install git=2.48.1",
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
        Vec::<Vec<&'static str>>::new()
    );
    assert_eq!(
        parse_args_and_compute_commands(FILE_CONTENT_2, FILE_CONTENT_2).unwrap(),
        Vec::<Vec<&'static str>>::new()
    );
}

fn parse_args_and_compute_commands(
    current_state_file_content: &'static str,
    target_state_file_content: &'static str,
) -> anyhow::Result<Vec<Vec<&'static str>>> {
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
