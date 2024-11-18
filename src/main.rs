use clap::Parser;
use futures::stream::StreamExt;
use reqwest::Client;
use rmk_config::toml_config::KeyboardTomlConfig;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::{env, process};
use zip::ZipArchive;

mod args;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::Args::parse();

    let keyboard_toml_config = read_keyboard_toml_config().unwrap();

    let project_name = keyboard_toml_config.keyboard.name.replace(" ", "_");
    let project_dir = env::current_dir().unwrap().join(&project_name);
    if let Err(e) = fs::create_dir_all(&project_dir) {
        eprintln!("Failed to create project directory {}: {}", project_name, e);
        process::exit(1);
    }

    // The following information is needed:
    // chip or board type
    // matrix type: normal or split?
    // USB or BLE
    // If nRF52840 -> S140 bootloader version?

    // TODO: download the corresponding project template to `project_dir`

    // TODO: Replace with actual GitHub repository information
    let user = "HaoboGu";
    let repo = "rmk-template";
    let branch = "feat/rework";
    let folder = "nrf52840";

    let url = format!(
        "https://github.com/{}/{}/archive/refs/heads/{}.zip",
        user, repo, branch
    );

    // Download project template
    download_with_progress(&url, &project_dir, folder).await?;

    // Copy keyboard.toml and vial.json to project_dir
    if let Err(e) = fs::copy(&args.keyboard_toml_path, project_dir.join("keyboard.toml")) {
        eprintln!("Failed to copy keyboard.toml to project directory: {}", e);
        process::exit(1);
    }

    if let Err(e) = fs::copy(&args.vial_json_path, project_dir.join("vial.json")) {
        eprintln!("Failed to copy vial.json to project directory: {}", e);
        process::exit(1);
    }

    Ok(())
}

/// Read the `keyboard.toml` configuration file
pub(crate) fn read_keyboard_toml_config() -> Result<KeyboardTomlConfig, String> {
    // Read the keyboard configuration file in the project root
    let s = match fs::read_to_string("keyboard.toml") {
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
) -> Result<(), Box<dyn std::error::Error>>
where
    P: AsRef<Path>,
{
    let output_path = output_path.as_ref();

    // Ensure the output path is clean
    if output_path.exists() {
        fs::remove_dir_all(output_path)?;
    }
    fs::create_dir_all(output_path)?;

    println!("Download project template...");

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

    println!("Download complete...");

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
            "The specified folder '{}' does not exist in the archive",
            folder
        )
        .into());
    }

    println!("Project created, path: {}", output_path.display());
    Ok(())
}
