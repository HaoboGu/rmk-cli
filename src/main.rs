use chip::get_chip_options;
use clap::Parser;
use futures::stream::StreamExt;
use inquire::ui::{Attributes, Color, RenderConfig, StyleSheet, Styled};
use inquire::{Select, Text};
use keyboard_toml::{parse_keyboard_toml, ProjectInfo};
use reqwest::Client;
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::{fs, process};
use zip::ZipArchive;

mod args;
mod chip;
mod keyboard_toml;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    inquire::set_global_render_config(get_render_config());
    let args = args::Args::parse();

    match args.command {
        args::Commands::Create {
            keyboard_toml_path,
            vial_json_path,
        } => create_project(keyboard_toml_path, vial_json_path).await,
        args::Commands::Init {
            project_name,
            chip,
            split,
        } => init_project(project_name, chip, split).await,
    }
}

async fn create_project(
    mut keyboard_toml_path: String,
    mut vial_json_path: String,
) -> Result<(), Box<dyn Error>> {
    // Inquire paths interactively is no argument is specified
    if keyboard_toml_path.is_empty() {
        keyboard_toml_path = Text::new("Path to keyboard.toml:")
            .with_default("./keyboard.toml")
            .prompt()?;
    }
    if vial_json_path.is_empty() {
        vial_json_path = Text::new("Path to vial.json")
            .with_default(&"./vial.json")
            .prompt()?
    }
    // Parse keyboard.toml to get project info
    let project_info = parse_keyboard_toml(&keyboard_toml_path)?;

    // Download corresponding project template
    download_project_template(&project_info).await?;

    // Copy keyboard.toml and vial.json to project_dir
    fs::copy(
        &keyboard_toml_path,
        project_info.target_dir.join("keyboard.toml"),
    )?;
    fs::copy(&vial_json_path, project_info.target_dir.join("vial.json"))?;

    // Post-process
    post_process(project_info)?;

    Ok(())
}

fn post_process(project_info: ProjectInfo) -> Result<(), Box<dyn Error>> {
    println!("Replacing project name placeholders...");
    let walker = walkdir::WalkDir::new(&project_info.target_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "toml"));
    Ok(for entry in walker {
        let path = entry.path();
        let content = fs::read_to_string(path)?;
        let new_content = content.replace("{{ project_name }}", &project_info.project_name);
        fs::write(path, new_content)?;
    })
}

async fn download_project_template(project_info: &ProjectInfo) -> Result<(), Box<dyn Error>> {
    let user = "HaoboGu";
    let repo = "rmk-template";
    let branch = "feat/rework";
    let url = format!(
        "https://github.com/{}/{}/archive/refs/heads/{}.zip",
        user, repo, branch
    );
    download_with_progress(&url, &project_info.target_dir, &project_info.remote_folder).await
}

async fn init_project(
    mut project_name: String,
    mut chip: String,
    split: bool,
) -> Result<(), Box<dyn Error>> {
    if project_name.is_empty() {
        project_name = Text::new("Project Name:").prompt()?;
    }
    if chip.is_empty() {
        chip = Select::new("Choose your microcontroller", get_chip_options())
            .prompt()?
            .to_string();
    }
    // Get project info from parameters
    let target_dir = PathBuf::from(&project_name);
    if let Err(e) = fs::create_dir_all(&target_dir) {
        eprintln!("Failed to create project directory {}: {}", project_name, e);
        process::exit(1);
    }
    let remote_folder = if split {
        format!("{}_{}", chip, "split")
    } else {
        chip.clone()
    };
    let project_info = ProjectInfo {
        project_name,
        target_dir,
        remote_folder,
    };

    // Download template
    download_project_template(&project_info).await?;

    // Post-process
    post_process(project_info)?;

    Ok(())
}

/// Download code from a GitHub repository link and extract it to the `repo` folder, using asynchronous download and a progress bar
///
/// # Parameters
/// - `download_url`: GitHub repository link
/// - `output_path`: Target extraction path
/// - `folder`: Specific subdirectory to extract
async fn download_with_progress<P>(
    download_url: &str,
    output_path: P,
    folder: &str,
) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path>,
{
    let output_path = output_path.as_ref();

    // Ensure the output path is clean
    if output_path.exists() {
        fs::remove_dir_all(output_path)?;
    }
    fs::create_dir_all(output_path)?;

    println!("Download project template for {}...", folder);

    // Send request and get response
    let client = Client::new();
    let response = client.get(download_url).send().await?;
    if !response.status().is_success() {
        return Err(format!("Download failed: {}", response.status()).into());
    }

    // Temporary file to store the downloaded content
    let temp_file_path = output_path.join("temp.zip");
    let mut temp_file = File::create(&temp_file_path)?;

    // Ensure the temporary file is cleaned up on error
    struct TempFileCleanup<'a> {
        path: &'a Path,
    }
    impl<'a> Drop for TempFileCleanup<'a> {
        fn drop(&mut self) {
            if self.path.exists() {
                if let Err(e) = fs::remove_file(self.path) {
                    eprintln!(
                        "Failed to remove temp file '{}': {}",
                        self.path.display(),
                        e
                    );
                }
            }
        }
    }
    let _cleanup_guard = TempFileCleanup {
        path: &temp_file_path,
    };

    // Stream response bytes and write to temp file
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        temp_file.write_all(&chunk)?;
    }

    // Open the downloaded ZIP file and extract
    let zip_file = File::open(&temp_file_path)?;
    let mut zip = ZipArchive::new(zip_file)?;

    let mut folder_found = false;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let file_name = file.enclosed_name().ok_or("Invalid file path")?;

        // Find the root directory from the ZIP file
        let segments: Vec<_> = file_name.iter().collect();
        if segments.len() > 1 && segments[1] == folder {
            folder_found = true;
            let relative_name = file_name.iter().skip(2).collect::<PathBuf>();
            let out_path = output_path.join(relative_name);

            if file.is_dir() {
                fs::create_dir_all(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = File::create(&out_path)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }
    }

    if !folder_found {
        return Err(format!(
            "The specified chip/board '{}' does not exist in the template",
            folder
        )
        .into());
    }

    println!("Project created, path: {}", output_path.display());
    Ok(())
}
fn get_render_config() -> RenderConfig<'static> {
    let mut render_config = RenderConfig::default();
    render_config.prompt_prefix = Styled::new("?").with_fg(Color::LightRed);

    render_config.error_message = render_config
        .error_message
        .with_prefix(Styled::new("❌").with_fg(Color::LightRed));

    render_config.answer = StyleSheet::new()
        .with_attr(Attributes::ITALIC)
        .with_fg(Color::LightGreen);

    render_config
}
