use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, exit};
use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use std::error::Error;
use regex::Regex;
use zip::read::ZipArchive;
use std::fs::File;

fn main() {
    // Set the working directory to the script's directory
    let script_directory = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    env::set_current_dir(&script_directory).expect("Failed to change directory");

    // Read the current app version
    let app_version = fs::read_to_string("version").expect("Failed to read the version file");
    let app_version = app_version.trim(); // Trim to remove any extra whitespace

    // GitHub repository URL
    let github_repo_url = "https://api.github.com/repos/Trenclik/KOK/releases";

    match check_for_updates(github_repo_url) {
        Some(latest_version) if latest_version != app_version => {
            println!("Latest version from GitHub: {}\nCurrent app version: {}", latest_version, app_version);
            update_app(&latest_version).unwrap_or_else(|e| {
                eprintln!("Update failed: {}", e);
                exit(1);
            });
        }
        _ => {
            println!("No updates available. Running the app...");
            run_application();
        }
    }
}

// Function to check for updates
fn check_for_updates(github_repo_url: &str) -> Option<String> {
    let client = Client::new();
    let response = client.get(github_repo_url)
        .headers(HeaderMap::new())
        .send()
        .ok()?;

    if response.status().is_success() {
        let releases_data: serde_json::Value = response.json().ok()?;
        let latest_release = releases_data.get(0)?;
        let latest_version = latest_release.get("tag_name")?.as_str()?;
        let regex = Regex::new(r"^v").unwrap();
        let cleaned_version = regex.replace(latest_version, "");
        Some(cleaned_version.to_string())
    } else {
        None
    }
}

// Function to update the app
fn update_app(latest_version: &str) -> Result<(), Box<dyn Error>> {
    println!("Updating...");

    // Download the update
    let download_url = format!("https://github.com/Trenclik/KOK/archive/{}.zip", latest_version);
    let response = reqwest::blocking::get(&download_url)?;
    let mut file = File::create("update.zip")?;
    file.write_all(&response.bytes()?)?;

    // Unpack the downloaded ZIP
    let file = File::open("update.zip")?;
    let mut zip_archive = ZipArchive::new(file)?;
    zip_archive.extract(".")?;

    let temp_dir = format!("kok-{}", latest_version.trim_start_matches('v'));
    fs::rename(&temp_dir, "temp")?;

    // Move files from temp to the root folder
    let temp_path = Path::new("temp");
    for entry in fs::read_dir(temp_path)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = source_path.file_name().map(|name| Path::new(".").join(name)).unwrap();

        if destination_path.exists() {
            fs::remove_file(&destination_path)?;
        }
        fs::copy(&source_path, &destination_path)?;
        println!("Replaced: {:?}", source_path.file_name().unwrap());
    }

    // Cleanup
    fs::remove_dir_all("temp")?;
    fs::remove_file("update.zip")?;

    // Restart the application
    run_application();
    Ok(())
}

// Function to run the main application
fn run_application() {
    if let Err(e) = Command::new("python").arg("submain_app.py").status() {
        eprintln!("Failed to run the application: {}", e);
        exit(1);
    }
}
