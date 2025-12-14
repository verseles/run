use anyhow::{Result, Context};
use std::env;
use std::process::Command;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use directories::ProjectDirs;

#[derive(Deserialize, Debug)]
struct Release {
    tag_name: String,
    body: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize, Debug)]
struct Asset {
    name: String,
    browser_download_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UpdateMetadata {
    updated_at: String,
    from_version: String,
    to_version: String,
    changelog_url: String,
    changelog_body: Option<String>,
}

pub fn spawn_auto_update() -> Result<()> {
    if env::var("RUN_NO_UPDATE").is_ok() {
        return Ok(());
    }

    let current_exe = env::current_exe()?;
    Command::new(current_exe)
        .arg("--internal-auto-update")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    Ok(())
}

pub async fn perform_update() -> Result<()> {
    // 1. Fetch latest release
    let client = reqwest::Client::new();
    let release: Release = client
        .get("https://api.github.com/repos/verseles/run/releases/latest")
        .header("User-Agent", "run-cli")
        .send()
        .await?
        .json()
        .await?;

    let current_version = env!("CARGO_PKG_VERSION");

    // Semver check
    let current = semver::Version::parse(current_version)?;
    let tag = release.tag_name.trim_start_matches('v');
    let remote = semver::Version::parse(tag)?;

    if remote <= current {
        return Ok(());
    }

    // 2. Download asset
    // Detect target based on runtime OS/Arch
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    let asset_name_part = match (os, arch) {
        ("linux", "x86_64") => "run-linux-x86_64",
        ("linux", "aarch64") => "run-linux-aarch64",
        ("macos", "x86_64") => "run-macos-x86_64",
        ("macos", "aarch64") => "run-macos-aarch64",
        ("windows", "x86_64") => "run-windows-x86_64.exe",
        _ => return Err(anyhow::anyhow!("Unsupported platform: {} {}", os, arch)),
    };

    let asset = release.assets.iter().find(|a| a.name.contains(asset_name_part) && (a.name.ends_with(".tar.gz") || a.name.ends_with(".zip")));

    if let Some(asset) = asset {
        // Download
        let bytes = client.get(&asset.browser_download_url)
            .send()
            .await?
            .bytes()
            .await?;

        // Extract and replace
        let temp_dir = tempfile::tempdir()?;
        let archive_path = temp_dir.path().join(&asset.name);
        fs::write(&archive_path, &bytes)?;

        let mut executable_path = PathBuf::new();

        if asset.name.ends_with(".zip") {
             let file = fs::File::open(&archive_path)?;
             let mut archive = zip::ZipArchive::new(file)?;
             // Assume bin is inside
             for i in 0..archive.len() {
                 let mut file = archive.by_index(i)?;
                 if file.name().contains("run.exe") {
                     let out_path = temp_dir.path().join(file.name());
                     let mut out_file = fs::File::create(&out_path)?;
                     std::io::copy(&mut file, &mut out_file)?;
                     executable_path = out_path;
                     break;
                 }
             }
        } else {
            // tar.gz
            let tar_gz = fs::File::open(&archive_path)?;
            let tar = flate2::read::GzDecoder::new(tar_gz);
            let mut archive = tar::Archive::new(tar);

            for entry in archive.entries()? {
                let mut entry = entry?;
                 let path = entry.path()?;
                 if path.to_string_lossy().contains("run") {
                      entry.unpack(temp_dir.path().join("run_new"))?;
                      executable_path = temp_dir.path().join("run_new");
                      break;
                 }
            }
        }

        if !executable_path.exists() {
             return Err(anyhow::anyhow!("Executable not found in archive"));
        }

        // Self replace
        self_replace::self_replace(&executable_path)?;

        // Save metadata
        if let Some(proj_dirs) = ProjectDirs::from("", "", "run") {
            let config_dir = proj_dirs.config_dir();
            fs::create_dir_all(config_dir)?;
            let update_file = config_dir.join("update.json");

            let metadata = UpdateMetadata {
                updated_at: chrono::Utc::now().to_rfc3339(),
                from_version: current_version.to_string(),
                to_version: remote.to_string(),
                changelog_url: format!("https://github.com/verseles/run/releases/tag/{}", release.tag_name),
                changelog_body: Some(release.body.lines().take(5).collect::<Vec<_>>().join("\n")),
            };

            let json = serde_json::to_string_pretty(&metadata)?;
            fs::write(update_file, json)?;
        }
    }

    Ok(())
}

pub fn check_for_update_notification() {
    if env::var("RUN_NO_UPDATE").is_ok() {
        return;
    }

    if let Some(proj_dirs) = ProjectDirs::from("", "", "run") {
        let update_file = proj_dirs.config_dir().join("update.json");
        if update_file.exists() {
            if let Ok(content) = fs::read_to_string(&update_file) {
                if let Ok(metadata) = serde_json::from_str::<UpdateMetadata>(&content) {
                     use owo_colors::OwoColorize;
                     println!("{}", format!("✓ run foi atualizado: v{} → v{}", metadata.from_version, metadata.to_version).green());
                     println!("");
                     println!("Mudanças principais:");
                     if let Some(body) = metadata.changelog_body {
                         println!("{}", body);
                     }
                     println!("");
                     println!("Ver changelog completo: {}", metadata.changelog_url);

                     // Delete file
                     let _ = fs::remove_file(update_file);
                }
            }
        }
    }
}
