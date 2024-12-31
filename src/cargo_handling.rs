use std::collections::BTreeMap;

use anyhow::bail;

use crate::command::{command, Command};
use crate::common::quote;

mod crate_name {
    crate::nonempty_str::newtype!(CrateName, error_msg = "empty crate name");
}
pub use crate_name::CrateName;

pub struct CargoInstall<'a>(CrateName<'a>, Command<'a>);

pub fn parse_line_with_cargo_install<'a>(
    left_trimmed_line: &'a str,
    cargo_map: &mut BTreeMap<CrateName<'a>, Command<'a>>,
) -> anyhow::Result<CargoInstall<'a>> {
    assert_eq!(left_trimmed_line.trim_start(), left_trimmed_line);
    assert!(left_trimmed_line.contains("cargo install "));
    let expected_suffix = "; \\";
    let Some(command_str) = left_trimmed_line.strip_suffix(expected_suffix) else {
        bail!(
            "line with \"cargo install \" but which does not end with {}",
            quote(expected_suffix)
        );
    };
    // `left_trimmed_line.contains("cargo install ")`` so `unwrap()` is OK.
    let crate_name_start_index =
        command_str.find("cargo install ").unwrap() + "cargo_install ".len();
    // `str::split` cannot return an empty iterator so `unwrap()` is OK.
    let crate_name_str = command_str[crate_name_start_index..].split(' ').next().unwrap();
    let crate_name = CrateName::from_str(crate_name_str)?;
    // The line is left timmed so `command_str` starts with a non-whitespace so `unwrap()` is OK.
    let command = Command::from_str(command_str).unwrap();
    if let Some(previous_command) = cargo_map.insert(crate_name, command.clone()) {
        bail!(
            "{} crate already installed in a previous line: the command was [{}]",
            quote(crate_name_str),
            previous_command.display()
        );
    }
    Ok(CargoInstall(crate_name, command))
}

pub fn compute_crate_removal_command<'a>(
    target_state_cargo_map: &BTreeMap<CrateName<'a>, Command<'a>>,
    current_state_action: &CargoInstall<'a>,
) -> Option<Command<'a>> {
    let crate_name = &current_state_action.0;
    (!target_state_cargo_map.contains_key(crate_name))
        .then(|| command!["cargo", "uninstall", crate_name.as_str()].unwrap())
}

pub fn compute_crate_install_or_update_command<'a>(
    current_state_cargo_map: &BTreeMap<CrateName<'a>, Command<'a>>,
    target_state_action: &CargoInstall<'a>,
) -> Option<Command<'a>> {
    let CargoInstall(crate_name, target_state_command) = target_state_action;
    if let Some(current_state_command) = current_state_cargo_map.get(crate_name) {
        if current_state_command == target_state_command {
            None
        } else {
            Some(target_state_command.concat_args(std::iter::once("--force".into())))
        }
    } else {
        Some(target_state_command.clone())
    }
}
