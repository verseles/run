use anyhow::Result;
use std::path::Path;
use crate::detectors::{Detector, Detection};

pub struct DotNetDetector;

impl Detector for DotNetDetector {
    fn detect(&self, path: &Path) -> Result<Option<Detection>> {
        // 17. .NET: *.csproj OR *.sln -> dotnet
        // We need to read dir entries to find wildcard extensions

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "csproj" || ext == "sln" {
                         return Ok(Some(Detection {
                            runner: "dotnet".to_string(),
                            command: "dotnet".to_string(),
                            lockfile: None,
                        }));
                    }
                }
            }
        }

        Ok(None)
    }
}
