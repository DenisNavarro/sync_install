use std::collections::BTreeMap;

use anyhow::Context as _;

use crate::cargo_handling::{
    compute_crate_install_or_update_command, compute_crate_removal_command,
    parse_line_with_cargo_install, CargoInstall, CrateName,
};
use crate::command::Command;
use crate::common::quote;
use crate::pixi_handling::{
    compute_recipe_install_or_update_command, compute_recipe_removal_command,
    parse_line_with_pixi_global_install, PixiGlobalInstall, Recipe, RecipeAndVersion,
};

pub struct State<'a> {
    ordered_actions: Vec<Action<'a>>,
    cargo_map: BTreeMap<CrateName<'a>, Command<'a>>,
    pixi_map: BTreeMap<Recipe<'a>, RecipeAndVersion<'a>>,
}

enum Action<'a> {
    CargoInstall(CargoInstall<'a>),
    PixiGlobalInstall(PixiGlobalInstall<'a>),
}

pub fn parse_state_from_file_content(file_content: &str) -> anyhow::Result<State> {
    let mut ordered_actions = Vec::new();
    let mut cargo_map = BTreeMap::new();
    let mut pixi_map = BTreeMap::new();
    for (index, line) in file_content.lines().enumerate() {
        let line_number = index + 1;
        let left_trimmed_line = line.trim_start();
        if left_trimmed_line.bytes().next() == Some(b'#') {
            continue;
        };
        (|| {
            if left_trimmed_line.contains("cargo install ") {
                let action = parse_line_with_cargo_install(left_trimmed_line, &mut cargo_map)?;
                ordered_actions.push(Action::CargoInstall(action));
            } else if left_trimmed_line.contains("pixi global install ") {
                let action = parse_line_with_pixi_global_install(left_trimmed_line, &mut pixi_map)?;
                ordered_actions.push(Action::PixiGlobalInstall(action));
            }
            anyhow::Ok(())
        })()
        .with_context(|| format!("failed to parse line {line_number}: {}", quote(line)))?;
    }
    Ok(State { ordered_actions, cargo_map, pixi_map })
}

// The current crate does not need to be optimized. So the return type of `compute_commands` could
// have been `Vec<Command>` with another `Command` type with owning strings so without lifetime.
// But I choosed to use iterators and lifetimes, just because it's more fun that way. :-)
pub fn compute_commands<'a, 'b>(
    current_state: &'b State<'a>,
    target_state: &'b State<'a>,
) -> impl Iterator<Item = Command<'a>> + use<'a, 'b> {
    itertools::chain![
        current_state.ordered_actions.iter().rev().filter_map(|action| match action {
            Action::CargoInstall(action) =>
                compute_crate_removal_command(&target_state.cargo_map, action),
            Action::PixiGlobalInstall(action) =>
                compute_recipe_removal_command(&target_state.pixi_map, *action),
        }),
        target_state.ordered_actions.iter().filter_map(|action| match action {
            Action::CargoInstall(action) =>
                compute_crate_install_or_update_command(&current_state.cargo_map, action),
            Action::PixiGlobalInstall(action) =>
                compute_recipe_install_or_update_command(&current_state.pixi_map, *action),
        }),
    ]
}
