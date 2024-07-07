use std::collections::BTreeMap;

use anyhow::{bail, Context};
use camino::Utf8Path;
use itertools::Either;

use crate::command::{command, Command};

mod nonempty_str_types {
    crate::nonempty_str::newtype!(PixiVersion, error_msg = "empty Pixi version");
    crate::nonempty_str::newtype!(Recipe, error_msg = "empty recipe");
    crate::nonempty_str::newtype!(RecipeAndVersion, error_msg = "neither recipe nor version");
}
pub use nonempty_str_types::{PixiVersion, Recipe, RecipeAndVersion};

#[derive(Clone, Copy)]
pub struct PixiGlobalInstall<'a>(Recipe<'a>, RecipeAndVersion<'a>);

pub fn parse_line_with_pixi_download_url<'a>(
    left_trimmed_line: &'a str,
    previous_pixi_version: Option<PixiVersion<'a>>,
) -> anyhow::Result<PixiVersion<'a>> {
    assert_eq!(left_trimmed_line.trim_start(), left_trimmed_line);
    assert!(left_trimmed_line.contains("https://github.com/prefix-dev/pixi/releases/download"));
    if let Some(previous_pixi_version) = previous_pixi_version {
        bail!(
            "Pixi download URL already in a previous line: the Pixi version was {}",
            previous_pixi_version.as_str()
        );
    }
    let expected_prefix = "\"https://github.com/prefix-dev/pixi/releases/download/v";
    let Some(striped_line) = left_trimmed_line.strip_prefix(expected_prefix) else {
        bail!("left trimmed line with Pixi download URL but which does not start with {expected_prefix:?}");
    };
    let expected_suffix = "/pixi-$(uname -m)-unknown-linux-musl\" \\";
    let Some(version_str) = striped_line.strip_suffix(expected_suffix) else {
        bail!("line with Pixi download URL but which does not end with {expected_suffix:?}");
    };
    let pixi_version = PixiVersion::from_str(version_str)?;
    Ok(pixi_version)
}

#[allow(clippy::option_if_let_else)]
pub fn compute_pixi_install_or_update_commands<'a>(
    current_state_pixi_version: Option<PixiVersion<'a>>,
    target_state_pixi_version: PixiVersion<'a>,
    home_path: &str,
    machine: &str,
) -> Either<std::option::IntoIter<Command<'a>>, <[Command<'static>; 3] as IntoIterator>::IntoIter> {
    if let Some(current_state_pixi_version) = current_state_pixi_version {
        if current_state_pixi_version == target_state_pixi_version {
            #[allow(clippy::iter_on_empty_collections)]
            Either::Left(None.into_iter())
        } else {
            let version = target_state_pixi_version.as_str();
            #[allow(clippy::iter_on_single_items)]
            Either::Left(
                Some(command!["pixi", "self-update", "--version", version].unwrap()).into_iter(),
            )
        }
    } else {
        let pixi_bin_path = Utf8Path::new(home_path).join(".pixi/bin").to_string();
        let pixi_path = Utf8Path::new(pixi_bin_path.as_str()).join("pixi").to_string();
        let version = target_state_pixi_version.as_str();
        let url = format!("https://github.com/prefix-dev/pixi/releases/download/v{version}/pixi-{machine}-unknown-linux-musl");
        Either::Right(
            [
                command!["mkdir", "-p", pixi_bin_path].unwrap(),
                command!["xh", "get", "--download", "--follow", url, "--output", pixi_path.clone()]
                    .unwrap(),
                command!["chmod", "+x", pixi_path].unwrap(),
            ]
            .into_iter(),
        )
    }
}

pub fn compute_pixi_removal_command(
    current_state_pixi_version: Option<PixiVersion>,
    target_state_pixi_version: Option<PixiVersion>,
    home_path: &str,
) -> Option<Command<'static>> {
    (current_state_pixi_version.is_some() && target_state_pixi_version.is_none()).then(|| {
        let pixi_path = Utf8Path::new(home_path).join(".pixi/bin/pixi").to_string();
        command!["rm", pixi_path].unwrap()
    })
}

pub fn parse_line_with_pixi_global_install<'a>(
    left_trimmed_line: &'a str,
    pixi_map: &mut BTreeMap<Recipe<'a>, RecipeAndVersion<'a>>,
) -> anyhow::Result<PixiGlobalInstall<'a>> {
    assert_eq!(left_trimmed_line.trim_start(), left_trimmed_line);
    assert!(left_trimmed_line.contains("pixi global install "));
    let expected_suffix = "; \\";
    let Some(command_str) = left_trimmed_line.strip_suffix(expected_suffix) else {
        bail!("line with \"pixi global install \" but which does not end with {expected_suffix:?}");
    };
    let expected_prefix = "pixi global install ";
    let Some(recipe_and_version_str) = command_str.strip_prefix(expected_prefix) else {
        bail!("left trimmed line with \"pixi global install \" but which does not start with {expected_prefix:?}");
    };
    let recipe_and_version = RecipeAndVersion::from_str(recipe_and_version_str)?;
    let recipe_end_index = recipe_and_version_str.find('=').context("'=' is missing")?;
    let recipe_str = &recipe_and_version_str[..recipe_end_index];
    let recipe = Recipe::from_str(recipe_str)?;
    if let Some(previous_recipe_and_version) = pixi_map.insert(recipe, recipe_and_version) {
        bail!(
            "{recipe_str:?} recipe already installed in a previous line: it was {}",
            previous_recipe_and_version.as_str()
        );
    }
    Ok(PixiGlobalInstall(recipe, recipe_and_version))
}

#[allow(clippy::option_if_let_else)]
pub fn compute_recipe_install_or_update_command<'a>(
    current_state_pixi_map: &BTreeMap<Recipe<'a>, RecipeAndVersion<'a>>,
    target_state_action: PixiGlobalInstall<'a>,
) -> Option<Command<'a>> {
    let PixiGlobalInstall(recipe, target_state_recipe_and_version) = target_state_action;
    if let Some(current_state_recipe_and_version) = current_state_pixi_map.get(&recipe) {
        if current_state_recipe_and_version == &target_state_recipe_and_version {
            None
        } else {
            let recipe_and_version = target_state_recipe_and_version.as_str();
            Some(command!["pixi", "global", "upgrade", recipe_and_version].unwrap())
        }
    } else {
        let recipe_and_version = target_state_recipe_and_version.as_str();
        Some(command!["pixi", "global", "install", recipe_and_version].unwrap())
    }
}

pub fn compute_recipe_removal_commands<'a, 'b>(
    current_state_pixi_map: &'b BTreeMap<Recipe<'a>, RecipeAndVersion<'a>>,
    target_state_pixi_map: &'b BTreeMap<Recipe<'a>, RecipeAndVersion<'a>>,
) -> impl Iterator<Item = Command<'a>> + 'b {
    current_state_pixi_map
        .keys()
        .copied()
        .filter(|recipe| !target_state_pixi_map.contains_key(recipe))
        .map(|recipe| command!["pixi", "global", "remove", recipe.as_str()].unwrap())
}
