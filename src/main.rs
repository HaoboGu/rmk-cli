use std::{env, fs, path::PathBuf};

use clap::Parser;
use rmk_config::toml_config::KeyboardTomlConfig;

fn default_keyboard_toml_path() -> PathBuf {
    env::current_dir().unwrap().join("keyboard.toml")
}

fn default_vial_json_path() -> PathBuf {
    env::current_dir().unwrap().join("vial.json")
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value=default_keyboard_toml_path().into_os_string())]
    keyboard_toml_path: PathBuf,

    /// Number of times to greet
    #[arg(short, long, default_value=default_vial_json_path().into_os_string())]
    vial_json_path: PathBuf,
}

pub(crate) fn read_keyboard_toml_config() -> Result<KeyboardTomlConfig, String> {
    // Read keyboard config file at project root
    let s = match fs::read_to_string("keyboard.toml") {
        Ok(s) => s,
        Err(e) => {
            let msg = format!("Read keyboard config file `keyboard.toml` error: {}", e);
            return Err(msg)
        }
    };

    // Parse keyboard config file content to `KeyboardTomlConfig`
    match toml::from_str(&s) {
        Ok(c) => Ok(c),
        Err(e) => {
            let msg = format!("Parse `keyboard.toml` error: {}", e.message());
            return Err(msg)
        }
    }
}

fn main()  {
    let args = Args::parse();

    let keyboard_toml_config = read_keyboard_toml_config().unwrap();

    // We need the following info:
    // Project name
    // chip or board
    // matrix type
    // USB or BLE
    // if nRF52840 -> S140 BL version?

    println!("Hello {:?}!", args.vial_json_path);
}
