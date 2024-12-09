use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new RMK project from keyboard.toml and vial.json
    Create {
        /// Path to keyboard.toml file
        #[arg(long, default_value = "")]
        keyboard_toml_path: String,

        /// Path to vial.json file
        #[arg(long, default_value = "")]
        vial_json_path: String,
    },

    /// Initialize a new RMK project with basic configuration
    Init {
        /// Name of the project
        #[arg(long, default_value = "")]
        project_name: String,

        /// Target chip (e.g., nrf52840)
        #[arg(long, default_value = "")]
        chip: String,

        /// Whether the keyboard is split
        #[arg(long, default_value_t = false)]
        split: bool,
    },
}
