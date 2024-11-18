use clap::Parser;
use std::{env, path::PathBuf};

fn default_keyboard_toml_path() -> PathBuf {
    env::current_dir().unwrap().join("keyboard.toml")
}

fn default_vial_json_path() -> PathBuf {
    env::current_dir().unwrap().join("vial.json")
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    /// Path to the `keyboard.toml` file
    #[arg(short, long, default_value=default_keyboard_toml_path().into_os_string())]
    pub(crate) keyboard_toml_path: PathBuf,

    /// Path to the `vial.json` file
    #[arg(short, long, default_value=default_vial_json_path().into_os_string())]
    pub(crate) vial_json_path: PathBuf,
}
