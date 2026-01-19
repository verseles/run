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

use super::{CommandSupport, CommandValidator, DetectedRunner, Ecosystem};
use std::path::Path;
use std::sync::Arc;

pub struct DotNetValidator;

impl CommandValidator for DotNetValidator {
    fn supports_command(&self, _working_dir: &Path, command: &str) -> CommandSupport {
        static DOTNET_BUILTIN: &[&str] = &[
            "build",
            "clean",
            "test",
            "run",
            "publish",
            "pack",
            "restore",
            "new",
            "add",
            "remove",
            "list",
            "nuget",
            "tool",
            "workload",
            "watch",
            "format",
            "help",
            "sln",
            "store",
            "msbuild",
            "vstest",
            "dev-certs",
            "fsi",
            "user-secrets",
            "ef",
        ];

        if DOTNET_BUILTIN.contains(&command) {
            return CommandSupport::Supported;
        }

        CommandSupport::NotSupported
    }
}

/// Detect .NET projects
/// Priority: 17
pub fn detect(dir: &Path) -> Vec<DetectedRunner> {
    let mut runners = Vec::new();
    let validator: Arc<dyn CommandValidator> = Arc::new(DotNetValidator);

    // Check for .csproj or .sln files
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "csproj" || ext == "sln" {
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    runners.push(DetectedRunner::with_validator(
                        "dotnet",
                        &file_name,
                        Ecosystem::DotNet,
                        17,
                        Arc::clone(&validator),
                    ));
                    break; // Only detect once
                }
            }
        }
    }

    runners
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_detect_csproj() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("MyApp.csproj")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "dotnet");
        assert_eq!(runners[0].detected_file, "MyApp.csproj");
    }

    #[test]
    fn test_detect_sln() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("MySolution.sln")).unwrap();

        let runners = detect(dir.path());
        assert_eq!(runners.len(), 1);
        assert_eq!(runners[0].name, "dotnet");
        assert_eq!(runners[0].detected_file, "MySolution.sln");
    }

    #[test]
    fn test_no_dotnet() {
        let dir = tempdir().unwrap();

        let runners = detect(dir.path());
        assert!(runners.is_empty());
    }
}
