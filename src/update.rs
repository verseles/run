// Copyright (C) 2025 Verseles
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

use crate::config::Config;
use crate::output;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

const GITHUB_REPO: &str = "verseles/run";
const UPDATE_TIMEOUT_SECS: u64 = 5;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub updated_at: DateTime<Utc>,
    pub from_version: String,
    pub to_version: String,
    pub changelog_url: String,
    pub changelog: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// Check if auto-update is disabled via environment variable
pub fn is_update_disabled() -> bool {
    env::var("RUN_NO_UPDATE").is_ok()
}

/// Get the current version of the CLI
pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Check for and display any pending update notifications
pub fn check_update_notification(quiet: bool) {
    if quiet {
        return;
    }

    let update_path = match Config::update_info_path() {
        Some(p) => p,
        None => return,
    };

    if !update_path.exists() {
        return;
    }

    // Read update info
    let content = match fs::read_to_string(&update_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let info: UpdateInfo = match serde_json::from_str(&content) {
        Ok(i) => i,
        Err(_) => {
            // Invalid file, remove it
            let _ = fs::remove_file(&update_path);
            return;
        }
    };

    // Check if update was recent (within 24 hours)
    let now = Utc::now();
    if now - info.updated_at > Duration::hours(24) {
        let _ = fs::remove_file(&update_path);
        return;
    }

    // Display update notification
    output::update_notification(
        &info.from_version,
        &info.to_version,
        info.changelog.as_deref(),
    );
    eprintln!();
    eprintln!("See full changelog: {}", info.changelog_url);
    eprintln!();

    // Remove the file after displaying
    let _ = fs::remove_file(&update_path);
}

/// Get the appropriate asset name for the current platform
fn get_asset_name() -> Option<String> {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    let platform = match (os, arch) {
        ("linux", "x86_64") => "run-linux-x86_64",
        ("linux", "aarch64") => "run-linux-aarch64",
        ("macos", "x86_64") => "run-macos-x86_64",
        ("macos", "aarch64") => "run-macos-aarch64",
        ("windows", "x86_64") => "run-windows-x86_64.exe",
        _ => return None,
    };

    Some(platform.to_string())
}

/// Spawn background update check
pub fn spawn_background_update() {
    if is_update_disabled() {
        return;
    }

    // Spawn detached process for update check
    // We use std::process::Command with specific flags to detach
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        use std::process::Command;

        let current_exe = match env::current_exe() {
            Ok(e) => e,
            Err(_) => return,
        };

        // Create a child process that will handle the update
        let _ = Command::new(&current_exe)
            .arg("--internal-update-check")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .process_group(0)
            .spawn();
    }

    #[cfg(windows)]
    {
        use std::process::Command;

        let current_exe = match env::current_exe() {
            Ok(e) => e,
            Err(_) => return,
        };

        let _ = Command::new(&current_exe)
            .arg("--internal-update-check")
            .creation_flags(0x00000008) // DETACHED_PROCESS
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
}

/// Perform the actual update check (called from background process)
pub async fn perform_update_check() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(UPDATE_TIMEOUT_SECS))
        .build()?;

    // Fetch latest release info
    let release: GitHubRelease = client
        .get(format!(
            "https://api.github.com/repos/{}/releases/latest",
            GITHUB_REPO
        ))
        .header("User-Agent", format!("run-cli/{}", current_version()))
        .send()
        .await?
        .json()
        .await?;

    // Parse versions
    let remote_version = release.tag_name.trim_start_matches('v');
    let local_version = current_version();

    let remote_semver = semver::Version::parse(remote_version)?;
    let local_semver = semver::Version::parse(local_version)?;

    if remote_semver <= local_semver {
        return Ok(()); // Already up to date
    }

    // Find the appropriate asset
    let asset_name = get_asset_name().ok_or("Unsupported platform")?;
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or("Asset not found for this platform")?;

    // Download the new binary
    let response = client.get(&asset.browser_download_url).send().await?;
    let bytes = response.bytes().await?;

    // Get current executable path
    let current_exe = env::current_exe()?;

    // Create a temporary file for the new binary
    let temp_path = current_exe.with_extension("new");

    // Write the new binary
    fs::write(&temp_path, bytes)?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o755))?;
    }

    // Atomic replace
    #[cfg(unix)]
    {
        fs::rename(&temp_path, &current_exe)?;
    }

    #[cfg(windows)]
    {
        // On Windows, we need to rename the current exe first
        let backup_path = current_exe.with_extension("old");
        let _ = fs::remove_file(&backup_path);
        fs::rename(&current_exe, &backup_path)?;
        fs::rename(&temp_path, &current_exe)?;
        let _ = fs::remove_file(&backup_path);
    }

    // Save update info
    let update_info = UpdateInfo {
        updated_at: Utc::now(),
        from_version: local_version.to_string(),
        to_version: remote_version.to_string(),
        changelog_url: release.html_url,
        changelog: release
            .body
            .map(|b| b.lines().take(5).collect::<Vec<_>>().join("\n")),
    };

    if let Some(path) = Config::update_info_path() {
        let _ = Config::ensure_config_dir();
        let _ = fs::write(path, serde_json::to_string_pretty(&update_info)?);
    }

    Ok(())
}

/// Perform a synchronous (blocking) update check
pub async fn perform_blocking_update(quiet: bool) -> Result<bool, Box<dyn std::error::Error>> {
    if !quiet {
        output::info("Checking for updates...");
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    // Fetch latest release info
    let release: GitHubRelease = client
        .get(format!(
            "https://api.github.com/repos/{}/releases/latest",
            GITHUB_REPO
        ))
        .header("User-Agent", format!("run-cli/{}", current_version()))
        .send()
        .await?
        .json()
        .await?;

    // Parse versions
    let remote_version = release.tag_name.trim_start_matches('v');
    let local_version = current_version();

    let remote_semver = semver::Version::parse(remote_version)?;
    let local_semver = semver::Version::parse(local_version)?;

    if remote_semver <= local_semver {
        if !quiet {
            output::success(&format!("Already up to date (v{})", local_version));
        }
        return Ok(false);
    }

    if !quiet {
        output::info(&format!(
            "Updating from v{} to v{}...",
            local_version, remote_version
        ));
    }

    // Find the appropriate asset
    let asset_name = get_asset_name().ok_or("Unsupported platform")?;
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or("Asset not found for this platform")?;

    // Download the new binary
    let response = client.get(&asset.browser_download_url).send().await?;
    let bytes = response.bytes().await?;

    // Get current executable path
    let current_exe = env::current_exe()?;

    // Create a temporary file for the new binary
    let temp_path = current_exe.with_extension("new");

    // Write the new binary
    fs::write(&temp_path, bytes)?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o755))?;
    }

    // Atomic replace
    #[cfg(unix)]
    {
        fs::rename(&temp_path, &current_exe)?;
    }

    #[cfg(windows)]
    {
        let backup_path = current_exe.with_extension("old");
        let _ = fs::remove_file(&backup_path);
        fs::rename(&current_exe, &backup_path)?;
        fs::rename(&temp_path, &current_exe)?;
        let _ = fs::remove_file(&backup_path);
    }

    if !quiet {
        output::success(&format!("Updated to v{}", remote_version));
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_version() {
        let version = current_version();
        assert!(!version.is_empty());
        assert!(semver::Version::parse(version).is_ok());
    }

    #[test]
    fn test_get_asset_name() {
        let asset = get_asset_name();
        // Should return Some on supported platforms
        #[cfg(any(
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "aarch64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "windows", target_arch = "x86_64")
        ))]
        assert!(asset.is_some());
    }
}
