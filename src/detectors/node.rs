use anyhow::{Result, Context};
use std::path::Path;
use crate::detectors::{Detector, Detection};
use which::which;
use owo_colors::OwoColorize;

pub struct NodeDetector;

impl Detector for NodeDetector {
    fn detect(&self, path: &Path) -> Result<Option<Detection>> {
        let mut candidates = Vec::new();

        // Check for all lockfiles
        if path.join("bun.lockb").exists() || (path.join("bun.lock").exists() && path.join("package.json").exists()) {
            candidates.push(("bun", "bun run", "bun.lockb"));
        }
        if path.join("pnpm-lock.yaml").exists() {
            candidates.push(("pnpm", "pnpm run", "pnpm-lock.yaml"));
        }
        if path.join("yarn.lock").exists() {
             candidates.push(("yarn", "yarn run", "yarn.lock"));
        }
        if path.join("package-lock.json").exists() {
             candidates.push(("npm", "npm run", "package-lock.json"));
        }

        // Resolution logic
        if candidates.is_empty() {
             // Fallback: package.json without lock -> npm
             if path.join("package.json").exists() {
                 return Ok(Some(Detection {
                    runner: "npm".to_string(),
                    command: "npm run".to_string(),
                    lockfile: None,
                }));
             }
             return Ok(None);
        }

        if candidates.len() == 1 {
            let (runner, command, lockfile) = candidates[0];
            return Ok(Some(Detection {
                runner: runner.to_string(),
                command: command.to_string(),
                lockfile: Some(lockfile.to_string()),
            }));
        }

        // Conflict! Multiple candidates.
        // Check which tools are installed.
        let mut installed_candidates = Vec::new();
        for (runner, command, lockfile) in &candidates {
            if which(runner).is_ok() {
                installed_candidates.push((*runner, *command, *lockfile));
            }
        }

        if installed_candidates.len() == 1 {
            let (runner, command, lockfile) = installed_candidates[0];
            // Warn user
            eprintln!(
                "{} Encontrados múltiplos lockfiles ({}), mas apenas {} está instalado. Usando {}.",
                "⚠ Aviso:".yellow(),
                candidates.iter().map(|c| c.2).collect::<Vec<_>>().join(", "),
                runner,
                runner
            );
             return Ok(Some(Detection {
                runner: runner.to_string(),
                command: command.to_string(),
                lockfile: Some(lockfile.to_string()),
            }));
        }

        if installed_candidates.len() > 1 {
            // Error!
             let msg = format!(
                "Erro: Detectados conflitos de lockfiles: {}.\nAmbas ferramentas ({}) estão instaladas globalmente.\nAção necessária: Remova o lockfile defasado ou use --ignore=<tool>.",
                candidates.iter().map(|c| c.2).collect::<Vec<_>>().join(", "),
                installed_candidates.iter().map(|c| c.0).collect::<Vec<_>>().join(", ")
            );
            return Err(anyhow::anyhow!(msg).context("Conflict detected"));
        }

        // None installed?
        let msg = format!(
            "Erro: Encontrados lockfiles ({}) mas nenhuma das ferramentas ({}) está instalada.",
             candidates.iter().map(|c| c.2).collect::<Vec<_>>().join(", "),
             candidates.iter().map(|c| c.0).collect::<Vec<_>>().join(", ")
        );
        return Err(anyhow::anyhow!(msg));
    }
}
