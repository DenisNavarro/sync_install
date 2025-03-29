mod cargo_handling;
mod command;
mod command_computing;
mod common;
mod nonempty_str;
mod pixi_handling;

// Remark about the unit tests in separate files:
// https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html#Assorted-Tricks

#[cfg(test)]
mod happy_path_tests;

#[cfg(test)]
mod parsing_error_tests;

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};
use clap::Parser;

use command::Command;
use command_computing::{compute_commands, parse_state_from_file_content};
use common::quote_path;

#[derive(Parser)]
#[command(version)]
#[clap(verbatim_doc_comment)]
/// Update what is installed by comparing two `Dockerfile`s.
///
/// For example, if the content of the `current_state` file is:
///
/// ```
/// FROM docker.io/library/rust:1.85.0-slim-bookworm
/// RUN set -eux; \
///     cargo install cargo-cache --version 0.8.3 --locked; \
///     cargo install cocogitto --version 6.2.0 --locked; \
///     cargo install fsays --version 0.3.0 --locked; \
///     cargo cache -r all
/// CMD ["/bin/bash"]
/// ```
///
/// and if the content of the `target_state` file is:
///
/// ```
/// FROM docker.io/library/rust:1.85.0-slim-bookworm
/// RUN set -eux; \
///     cargo install cargo-cache --version 0.8.3; \
///     cargo install cocogitto --version 6.2.0 --locked; \
///     cargo install pixi --git https://github.com/prefix-dev/pixi.git --tag v0.41.4 --locked; \
///     cargo cache -r all
/// CMD ["/bin/bash"]
/// ```
///
/// then the output of `sync_install current_state target_state` will be:
///
/// ```
/// This is a dry run. Add the --go option to execute the below command(s).
/// ---> [cargo uninstall fsays]
/// ---> [cargo install cargo-cache --version 0.8.3 --force]
/// ---> [cargo install pixi --git https://github.com/prefix-dev/pixi.git --tag v0.41.4 --locked]
/// ```
///
/// Warning: This program is limited to a few use cases of its author and the
/// format of the `Dockerfile` contents must follow some arbitrary rules.
///
/// Tip: Update by comparing a `Dockerfile` to its state in the Git index:
///
/// ```
/// sync_install <(git show :./Dockerfile) Dockerfile
/// sync_install <(git show :./Dockerfile) Dockerfile --go
/// ```
struct Cli {
    /// Dockerfile
    current_state_file_path: PathBuf,
    /// Dockerfile
    target_state_file_path: PathBuf,
    /// Cancel the dry run
    #[arg(long)]
    go: bool,
}

macro_rules! my_writeln {
    ($($x:expr),+ $(,)?) => {
        writeln!(std::io::stdout(), $($x),+).context("failed to write to stdout")
    };
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let current_state_file_path = &cli.current_state_file_path;
    let target_state_file_path = &cli.target_state_file_path;
    let dry_run = !cli.go;
    if dry_run {
        my_writeln!("This is a dry run. Add the --go option to execute the below command(s).")?;
    }
    let data = get_input_data(current_state_file_path, target_state_file_path)?;
    let current_state = parse_state_from_file_content(&data.current_state_file_content)
        .with_context(|| {
            format!("failed to parse the content of {}", quote_path(current_state_file_path))
        })?;
    let target_state = parse_state_from_file_content(&data.target_state_file_content)
        .with_context(|| {
            format!("failed to parse the content of {}", quote_path(target_state_file_path))
        })?;
    compute_commands(&current_state, &target_state)
        .try_for_each(|command| print_and_execute(&command, dry_run))
}

struct InputData {
    current_state_file_content: String,
    target_state_file_content: String,
}

fn get_input_data(
    current_state_file_path: &Path,
    target_state_file_path: &Path,
) -> anyhow::Result<InputData> {
    let current_state_file_content = fs::read_to_string(current_state_file_path)
        .with_context(|| format!("failed to read {}", quote_path(current_state_file_path)))?;
    let target_state_file_content = fs::read_to_string(target_state_file_path)
        .with_context(|| format!("failed to read {}", quote_path(target_state_file_path)))?;
    Ok(InputData { current_state_file_content, target_state_file_content })
}

fn print_and_execute(command: &Command, dry_run: bool) -> anyhow::Result<()> {
    my_writeln!("---> [{}]", command.display())?;
    if !dry_run {
        execute(command)?;
    }
    Ok(())
}

fn execute(command: &Command) -> anyhow::Result<()> {
    let (program, args) = command.split_program_and_args();
    std::process::Command::new(program)
        .args(args)
        .status()
        .context("failed to execute process")
        .and_then(|status| {
            if !status.success() {
                bail!("error status: {status}");
            }
            Ok(())
        })
        .with_context(|| format!("failed to run [{}]", command.display()))
}
