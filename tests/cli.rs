use anyhow::{ensure, Context};
use camino::Utf8PathBuf;
use cargo_metadata::MetadataCommand;

const EXPECTED_RESULT: &str = r"This is a dry run. Add the --go option to execute the below command(s).
---> [cargo uninstall genact]
---> [cargo install cargo-cache --version 0.8.3 --force]
---> [cargo install xh --version 0.22.0 --locked]
";

#[test]
fn example_from_the_cli_help_and_the_readme() -> anyhow::Result<()> {
    let cargo_target_dir = get_cargo_target_dir()?;
    let output = std::process::Command::new(cargo_target_dir.join("debug/sync_install"))
        .arg("tests/current_state_from_readme")
        .arg("tests/target_state_from_readme")
        .output()
        .context("failed to execute process")?;
    let status = output.status;
    ensure!(status.success(), "error status: {status}");
    let stdout = String::from_utf8(output.stdout).context("non-UTF8 command output")?;
    assert_eq!(stdout, EXPECTED_RESULT);
    Ok(())
}

fn get_cargo_target_dir() -> anyhow::Result<Utf8PathBuf> {
    let cmd = MetadataCommand::new();
    let metadata = cmd.exec().with_context(|| format!("failed to execute command {cmd:?}"))?;
    Ok(metadata.target_directory)
}
