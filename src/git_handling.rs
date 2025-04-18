use std::collections::HashMap;

use anyhow::bail;

use crate::command::{Command, command};
use crate::common::quote;

mod nonempty_str_types {
    crate::nonempty_str::newtype!(GitConfigOption, error_msg = "empty option");
    crate::nonempty_str::newtype!(GitConfigValue, error_msg = "empty value");
}
pub use nonempty_str_types::{GitConfigOption, GitConfigValue};

#[derive(Clone, Copy)]
pub struct GitConfigSetGlobal<'a>(GitConfigOption<'a>, GitConfigValue<'a>);

pub fn parse_stripped_line_with_git_config_set_global<'a>(
    stripped_line: &'a str,
    git_map: &mut HashMap<GitConfigOption<'a>, GitConfigValue<'a>>,
) -> anyhow::Result<GitConfigSetGlobal<'a>> {
    let expected_suffix = "; \\";
    let Some(option_and_value) = stripped_line.strip_suffix(expected_suffix) else {
        bail!(
            "line with \"git config set --global \" but which does not end with {}",
            quote(expected_suffix)
        );
    };
    let Some((option_str, mut value_str)) = option_and_value.split_once(' ') else {
        bail!("{} git global option without value", quote(option_and_value));
    };
    let option = GitConfigOption::from_str(option_str)?;
    if let Some(("'", rest)) = value_str.split_at_checked(1) {
        if let Some(new_value_str) = rest.strip_suffix('\'') {
            value_str = new_value_str;
        } else {
            bail!("missing ending apostrophe in {}", quote(value_str));
        }
    }
    let value = GitConfigValue::from_str(value_str)?;
    if let Some(previous_value) = git_map.insert(option, value) {
        bail!(
            "{} git global option already set in a previous line: the value was {}",
            quote(option_str),
            quote(previous_value.as_str()),
        );
    }
    Ok(GitConfigSetGlobal(option, value))
}

pub fn compute_git_global_config_removal_command<'a>(
    target_state_git_map: &HashMap<GitConfigOption<'a>, GitConfigValue<'a>>,
    current_state_action: GitConfigSetGlobal<'a>,
) -> Option<Command<'a>> {
    let option = &current_state_action.0;
    (!target_state_git_map.contains_key(option))
        .then(|| command!["git", "config", "unset", "--global", option.as_str()].unwrap())
}

pub fn compute_git_global_config_set_or_update_command<'a>(
    current_state_git_map: &HashMap<GitConfigOption<'a>, GitConfigValue<'a>>,
    target_state_action: GitConfigSetGlobal<'a>,
) -> Option<Command<'a>> {
    let GitConfigSetGlobal(option, target_state_value) = target_state_action;
    current_state_git_map
        .get(&option)
        .is_none_or(|current_state_value| current_state_value != &target_state_value)
        .then(|| {
            let value = target_state_value.as_str();
            command!["git", "config", "set", "--global", option.as_str(), value].unwrap()
        })
}
