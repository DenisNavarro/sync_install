mod cargo_handling;
mod command;
mod command_computing;
mod nonempty_str;
mod pixi_handling;

// Remark about the unit tests in separate files:
// https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html#Assorted-Tricks

#[cfg(test)]
mod happy_path_tests;

#[cfg(test)]
mod parsing_error_tests;

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use clap::Parser;
use home::home_dir; // std::env::home_dir is deprecated since Rust 1.29.0.
use platform_info::{PlatformInfo, PlatformInfoAPI, UNameAPI};

use command::Command;
use command_computing::{compute_commands, parse_state_from_file_content};

#[derive(Parser)]
#[command(version)]
#[clap(verbatim_doc_comment)]
/// Update what is installed by comparing two `Dockerfile`s.
///
/// For example, if the content of the `current_state` file is:
///
/// ```
/// FROM docker.io/library/rust:1.78.0-slim-bookworm
/// RUN set -eux; \
///     cargo install bat --version 0.24.0 --locked; \
///     cargo install cargo-cache --version 0.8.3 --locked; \
///     cargo install genact --version 1.4.2; \
///     cargo cache -r all
/// CMD ["/bin/bash"]
/// ```
///
/// and if the content of the `target_state` file is:
///
/// ```
/// FROM docker.io/library/rust:1.78.0-slim-bookworm
/// RUN set -eux; \
///     cargo install bat --version 0.24.0 --locked; \
///     cargo install cargo-cache --version 0.8.3; \
///     cargo install xh --version 0.22.0 --locked; \
///     cargo cache -r all
/// CMD ["/bin/bash"]
/// ```
///
/// then the output of `sync_install current_state target_state` will be:
///
/// ```
/// This is a dry run. Add the --go option to execute the below command(s).
/// ---> [cargo uninstall genact]
/// ---> [cargo install cargo-cache --version 0.8.3 --force]
/// ---> [cargo install xh --version 0.22.0 --locked]
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
    let machine = get_machine(&data)?; // example: "x86_64"
    let current_state = parse_state_from_file_content(&data.current_state_file_content)
        .with_context(|| format!("failed to parse the content of {current_state_file_path:?}"))?;
    let target_state = parse_state_from_file_content(&data.target_state_file_content)
        .with_context(|| format!("failed to parse the content of {target_state_file_path:?}"))?;
    let mut commands = compute_commands(&current_state, &target_state, &data.home_path, machine);
    commands.try_for_each(|command| print_and_execute(&command, dry_run))
}

struct InputData {
    current_state_file_content: String,
    target_state_file_content: String,
    home_path: String,
    platform_info: PlatformInfo,
}

fn get_input_data(
    current_state_file_path: &Path,
    target_state_file_path: &Path,
) -> anyhow::Result<InputData> {
    let current_state_file_content = fs::read_to_string(current_state_file_path)
        .with_context(|| format!("failed to read {current_state_file_path:?}"))?;
    let target_state_file_content = fs::read_to_string(target_state_file_path)
        .with_context(|| format!("failed to read {target_state_file_path:?}"))?;
    let home_path = home_dir().context("failed to get the home directory path")?;
    let home_path = match home_path.into_os_string().into_string() {
        Ok(home_path) => home_path,
        Err(home_path) => bail!("non UTF-8 home directory path: {:?}", home_path.to_string_lossy()),
    };
    let Ok(platform_info) = PlatformInfo::new() else {
        bail!("failed to determine platform info");
    };
    Ok(InputData {
        current_state_file_content,
        target_state_file_content,
        home_path,
        platform_info,
    })
}

fn get_machine(data: &InputData) -> anyhow::Result<&str> {
    let machine = data.platform_info.machine();
    machine.to_str().with_context(|| format!("non UTF-8 machine: {:?}", machine.to_string_lossy()))
}

fn print_and_execute(command: &Command, dry_run: bool) -> anyhow::Result<()> {
    my_writeln!("---> [{}]", command.format())?;
    if !dry_run {
        execute(command)?;
    }
    Ok(())
}

fn execute(command: &Command) -> anyhow::Result<()> {
    let (program, args) = command.split_program_and_args();
    std::process::Command::new(program.as_ref())
        .args(args.iter().map(AsRef::as_ref))
        .status()
        .context("failed to execute process")
        .and_then(|status| {
            if !status.success() {
                bail!("error status: {status}");
            }
            Ok(())
        })
        .with_context(|| format!("failed to run [{}]", command.format()))
}
