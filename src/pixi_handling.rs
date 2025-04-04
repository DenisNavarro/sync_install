use std::collections::HashMap;

use anyhow::{Context as _, bail};

use crate::command::{Command, command};
use crate::common::quote;

mod nonempty_str_types {
    crate::nonempty_str::newtype!(Recipe, error_msg = "empty recipe");
    crate::nonempty_str::newtype!(RecipeAndVersion, error_msg = "neither recipe nor version");
}
pub use nonempty_str_types::{Recipe, RecipeAndVersion};

#[derive(Clone, Copy)]
pub struct PixiGlobalInstall<'a>(Recipe<'a>, RecipeAndVersion<'a>);

pub fn parse_stripped_line_with_pixi_global_install<'a>(
    stripped_line: &'a str,
    pixi_map: &mut HashMap<Recipe<'a>, RecipeAndVersion<'a>>,
) -> anyhow::Result<PixiGlobalInstall<'a>> {
    let expected_suffix = "; \\";
    let Some(recipe_and_version_str) = stripped_line.strip_suffix(expected_suffix) else {
        bail!(
            "line with \"pixi global install \" but which does not end with {}",
            quote(expected_suffix)
        );
    };
    let recipe_and_version = RecipeAndVersion::from_str(recipe_and_version_str)?;
    let recipe_end_index = recipe_and_version_str.find('=').context("'=' is missing")?;
    let recipe_str = &recipe_and_version_str[..recipe_end_index];
    let recipe = Recipe::from_str(recipe_str)?;
    if let Some(previous_recipe_and_version) = pixi_map.insert(recipe, recipe_and_version) {
        bail!(
            "{} recipe already installed in a previous line: it was {}",
            quote(recipe_str),
            previous_recipe_and_version.as_str()
        );
    }
    Ok(PixiGlobalInstall(recipe, recipe_and_version))
}

pub fn compute_recipe_removal_command<'a>(
    target_state_pixi_map: &HashMap<Recipe<'a>, RecipeAndVersion<'a>>,
    current_state_action: PixiGlobalInstall<'a>,
) -> Option<Command<'a>> {
    let recipe = &current_state_action.0;
    (!target_state_pixi_map.contains_key(recipe))
        .then(|| command!["pixi", "global", "uninstall", recipe.as_str()].unwrap())
}

pub fn compute_recipe_install_or_update_command<'a>(
    current_state_pixi_map: &HashMap<Recipe<'a>, RecipeAndVersion<'a>>,
    target_state_action: PixiGlobalInstall<'a>,
) -> Option<Command<'a>> {
    let PixiGlobalInstall(recipe, target_state_recipe_and_version) = target_state_action;
    current_state_pixi_map
        .get(&recipe)
        .is_none_or(|current_state_recipe_and_version| {
            current_state_recipe_and_version != &target_state_recipe_and_version
        })
        .then(|| {
            let recipe_and_version = target_state_recipe_and_version.as_str();
            command!["pixi", "global", "install", recipe_and_version].unwrap()
        })
}
