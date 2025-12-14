use anyhow::{Result, Context};
use std::path::Path;
use crate::detectors::{Detector, Detection};
use which::which;
use owo_colors::OwoColorize;

pub struct PythonDetector;

impl Detector for PythonDetector {
    fn detect(&self, path: &Path) -> Result<Option<Detection>> {
        let mut candidates = Vec::new();

        if path.join("uv.lock").exists() {
            candidates.push(("uv", "uv run", "uv.lock"));
        }
        if path.join("poetry.lock").exists() {
            candidates.push(("poetry", "poetry run", "poetry.lock"));
        }
        if path.join("Pipfile.lock").exists() {
            candidates.push(("pipenv", "pipenv run", "Pipfile.lock"));
        }

        // Resolution
        if candidates.is_empty() {
             // Fallback
             if path.join("requirements.txt").exists() || path.join("pyproject.toml").exists() {
                 return Ok(Some(Detection {
                    runner: "pip".to_string(),
                    command: "python -m".to_string(),
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

        // Conflict!
        let mut installed_candidates = Vec::new();
        for (runner, command, lockfile) in &candidates {
            if which(runner).is_ok() {
                installed_candidates.push((*runner, *command, *lockfile));
            }
        }

        if installed_candidates.len() == 1 {
             let (runner, command, lockfile) = installed_candidates[0];
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
             let msg = format!(
                "Erro: Detectados conflitos de lockfiles: {}.\nAmbas ferramentas ({}) estão instaladas globalmente.\nAção necessária: Remova o lockfile defasado ou use --ignore=<tool>.",
                candidates.iter().map(|c| c.2).collect::<Vec<_>>().join(", "),
                installed_candidates.iter().map(|c| c.0).collect::<Vec<_>>().join(", ")
            );
            return Err(anyhow::anyhow!(msg).context("Conflict detected"));
        }

         let msg = format!(
            "Erro: Encontrados lockfiles ({}) mas nenhuma das ferramentas ({}) está instalada.",
             candidates.iter().map(|c| c.2).collect::<Vec<_>>().join(", "),
             candidates.iter().map(|c| c.0).collect::<Vec<_>>().join(", ")
        );
        return Err(anyhow::anyhow!(msg));
    }
}
