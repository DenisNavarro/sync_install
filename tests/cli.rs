use anyhow::{Context as _, ensure};

const EXPECTED_OUTPUT: &str =
    "This is a dry run. Add the --go option to execute the below command(s).
---> [cargo uninstall fsays]
---> [cargo install cargo-cache --version 0.8.3 --force]
---> [cargo install pixi --git https://github.com/prefix-dev/pixi.git --tag v0.68.0 --locked]
";

#[test]
fn example_from_the_cli_help_and_the_readme() -> anyhow::Result<()> {
    let output = std::process::Command::new("cargo")
        .arg("run")
        .arg("-q")
        .arg("--")
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
