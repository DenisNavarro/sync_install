use anyhow::{Context as _, ensure};
use camino::Utf8PathBuf;
use cargo_metadata::MetadataCommand;

const EXPECTED_OUTPUT: &str =
    "This is a dry run. Add the --go option to execute the below command(s).
---> [cargo uninstall fsays]
---> [cargo install cargo-cache --version 0.8.3 --force]
---> [cargo install pixi --git https://github.com/prefix-dev/pixi.git --tag v0.41.4 --locked]
";

#[test]
fn example_from_the_cli_help_and_the_readme() -> anyhow::Result<()> {
    let cargo_target_dir =
        get_cargo_target_dir().context("failed to get cargo target directory")?;
    let output = std::process::Command::new(cargo_target_dir.join("debug/sync_install"))
        .arg("dockerfiles/current_state_from_readme")
        .arg("dockerfiles/target_state_from_readme")
        .output()
        .context("failed to execute process")?;
    let status = output.status;
    ensure!(status.success(), "error status: {status}");
    let stdout = String::from_utf8(output.stdout).context("non-UTF8 command output")?;
    assert_eq!(stdout, EXPECTED_OUTPUT);
    Ok(())
}

fn get_cargo_target_dir() -> anyhow::Result<Utf8PathBuf> {
    let metadata = MetadataCommand::new().exec().context("failed to execute metadata command")?;
    Ok(metadata.target_directory)
}
