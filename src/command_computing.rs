use std::collections::BTreeMap;

use anyhow::Context;
use itertools::Either;

use crate::cargo_handling::{
    compute_crate_install_or_update_command, compute_crate_removal_commands,
    parse_line_with_cargo_install, CargoInstall, CrateName,
};
use crate::command::Command;
use crate::pixi_handling::{
    compute_pixi_install_or_update_commands, compute_pixi_removal_command,
    compute_recipe_install_or_update_command, compute_recipe_removal_commands,
    parse_line_with_pixi_download_url, parse_line_with_pixi_global_install, PixiGlobalInstall,
    PixiVersion, Recipe, RecipeAndVersion,
};

pub struct State<'a> {
    ordered_actions: Vec<Action<'a>>,
    cargo_map: BTreeMap<CrateName<'a>, Command<'a>>,
    pixi_version: Option<PixiVersion<'a>>,
    pixi_map: BTreeMap<Recipe<'a>, RecipeAndVersion<'a>>,
}

enum Action<'a> {
    CargoInstall(CargoInstall<'a>),
    PixiDownload(PixiVersion<'a>),
    PixiGlobalInstall(PixiGlobalInstall<'a>),
}

pub fn parse_state_from_file_content(file_content: &str) -> anyhow::Result<State> {
    let mut ordered_actions = Vec::new();
    let mut cargo_map = BTreeMap::new();
    let mut pixi_version = None;
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
            } else if left_trimmed_line
                .contains("https://github.com/prefix-dev/pixi/releases/download")
            {
                let version = parse_line_with_pixi_download_url(left_trimmed_line, pixi_version)?;
                pixi_version = Some(version);
                ordered_actions.push(Action::PixiDownload(version));
            } else if left_trimmed_line.contains("pixi global install ") {
                let action = parse_line_with_pixi_global_install(left_trimmed_line, &mut pixi_map)?;
                ordered_actions.push(Action::PixiGlobalInstall(action));
            }
            anyhow::Ok(())
        })()
        .with_context(|| format!("failed to parse line {line_number}: {line:?}"))?;
    }
    Ok(State { ordered_actions, cargo_map, pixi_version, pixi_map })
}

// The current crate does not need to be optimized. So the return type of `compute_commands` could
// have been `Vec<Command>` with another `Command` type with owning strings so without lifetime.
// But I choosed to use iterators and lifetimes, just because it's more fun that way. :-)
pub fn compute_commands<'a, 'b>(
    current_state: &'b State<'a>,
    target_state: &'b State<'a>,
    home_path: &'b str,
    machine: &'b str,
) -> impl Iterator<Item = Command<'a>> + 'b {
    itertools::chain![
        compute_crate_removal_commands(&current_state.cargo_map, &target_state.cargo_map),
        compute_recipe_removal_commands(&current_state.pixi_map, &target_state.pixi_map),
        compute_pixi_removal_command(
            current_state.pixi_version,
            target_state.pixi_version,
            home_path
        ),
        target_state.ordered_actions.iter().flat_map(|action| match action {
            Action::CargoInstall(action) => Either::Left(
                compute_crate_install_or_update_command(&current_state.cargo_map, action)
                    .into_iter(),
            ),
            Action::PixiDownload(pixi_version) => {
                compute_pixi_install_or_update_commands(
                    current_state.pixi_version,
                    *pixi_version,
                    home_path,
                    machine,
                )
            }
            Action::PixiGlobalInstall(action) => Either::Left(
                compute_recipe_install_or_update_command(&current_state.pixi_map, *action)
                    .into_iter(),
            ),
        }),
    ]
}
