use rmk_config::toml_config::KeyboardTomlConfig;
use std::{env, fs, path::{Path, PathBuf}, process};

use crate::chip::get_board_chip_map;

/// All info needed to create a RMK project
pub(crate) struct ProjectInfo {
    /// Project name
    pub(crate) project_name: String,
    /// Local directory of created RMK project
    pub(crate) target_dir: PathBuf,
    /// Remote folder name which contains the template
    pub(crate) remote_folder: String,
}

/// Parse `keyboard.toml`, get all needed project info for creating a new RMK project
pub(crate) fn parse_keyboard_toml(
    keyboard_toml: &String,
) -> Result<ProjectInfo, Box<dyn std::error::Error>> {
    let keyboard_toml_config = read_keyboard_toml_config(keyboard_toml)?;

    let project_name = keyboard_toml_config.keyboard.name.replace(" ", "_");
    let project_dir = env::current_dir()?.join(&project_name);

    if let Err(e) = fs::create_dir_all(&project_dir) {
        eprintln!("Failed to create project directory {}: {}", project_name, e);
        process::exit(1);
    }

    // Check keyboard.toml
    let chip = match (
        keyboard_toml_config.keyboard.board.as_deref(),
        keyboard_toml_config.keyboard.chip.as_deref(),
    ) {
        (None, None) => {
            Err("Either 'board' or 'chip' must be specified in keyboard.toml".to_string())
        }
        (Some(board), None) => {
            let map = get_board_chip_map();
            map.get(board.to_lowercase().as_str())
                .map(|chip| chip.to_string())
                .ok_or_else(|| format!("Unsupported board '{}'", board))
        }
        (None, Some(chip)) => Ok(chip.to_string().to_lowercase()),
        (Some(_), Some(_)) => {
            Err("'board' and 'chip' cannot both be specified in keyboard.toml".to_string())
        }
    }?;

    let matrix_type = match (keyboard_toml_config.matrix, keyboard_toml_config.split) {
        (None, None) => {
            Err("Either 'matrix' or 'split' section must be specified in keyboard.toml".to_string())
        }
        (None, Some(_)) => Ok("split".to_string()),
        (Some(_), None) => Ok("normal".to_string()),
        (Some(_), Some(_)) => {
            Err("'matrix' and 'split' cannot both be specified in keyboard.toml".to_string())
        }
    }?;

    let folder = if matrix_type == "split" {
        format!("{}_{}", chip, matrix_type)
    } else {
        chip
    };

    Ok(ProjectInfo {
        project_name,
        target_dir: project_dir,
        remote_folder: folder,
    })
}

/// Read the `keyboard.toml` configuration file
pub(crate) fn read_keyboard_toml_config<P: AsRef<Path>>(
    keyboard_toml: P,
) -> Result<KeyboardTomlConfig, String> {
    // Read the keyboard configuration file in the project root
    let s = match fs::read_to_string(keyboard_toml) {
        Ok(s) => s,
        Err(e) => {
            let msg = format!("Failed to read `keyboard.toml` configuration file: {}", e);
            return Err(msg);
        }
    };

    // Parse the configuration file content into a `KeyboardTomlConfig` struct
    match toml::from_str(&s) {
        Ok(c) => Ok(c),
        Err(e) => {
            let msg = format!("Failed to parse `keyboard.toml`: {}", e.message());
            return Err(msg);
        }
    }
}
