use crate::flags::Manager;
use anyhow::{Context, Result};
use directories_next::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub defaults: Defaults,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Defaults {
    pub javascript: Option<Manager>,
    pub python: Option<Manager>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub manager: Option<Manager>,
}

pub fn global_config_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "add").map(|p| p.config_dir().join("config.toml"))
}

pub fn load_global() -> Result<GlobalConfig> {
    let Some(path) = global_config_path() else {
        return Ok(GlobalConfig::default());
    };
    if !path.is_file() {
        return Ok(GlobalConfig::default());
    }
    let text = fs::read_to_string(&path)
        .with_context(|| format!("reading global config at {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parsing global config at {}", path.display()))
}

pub fn find_project_config(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        let candidate = dir.join(".addrc.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
        current = dir.parent();
    }
    None
}

pub fn load_project(start: &Path) -> Result<Option<ProjectConfig>> {
    let Some(path) = find_project_config(start) else {
        return Ok(None);
    };
    let text = fs::read_to_string(&path)
        .with_context(|| format!("reading .addrc.toml at {}", path.display()))?;
    let cfg: ProjectConfig = toml::from_str(&text)
        .with_context(|| format!("parsing .addrc.toml at {}", path.display()))?;
    Ok(Some(cfg))
}

pub fn write_project_config(dir: &Path, cfg: &ProjectConfig) -> Result<PathBuf> {
    let path = dir.join(".addrc.toml");
    let text = toml::to_string_pretty(cfg).context("serialising .addrc.toml")?;
    fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn project_config_round_trips() {
        let dir = tempdir().unwrap();
        let cfg = ProjectConfig {
            manager: Some(Manager::Pnpm),
        };
        let path = write_project_config(dir.path(), &cfg).unwrap();
        assert!(path.is_file());
        let loaded = load_project(dir.path()).unwrap().unwrap();
        assert_eq!(loaded.manager, Some(Manager::Pnpm));
    }

    #[test]
    fn project_config_walks_up() {
        let dir = tempdir().unwrap();
        write_project_config(
            dir.path(),
            &ProjectConfig {
                manager: Some(Manager::Bun),
            },
        )
        .unwrap();
        let sub = dir.path().join("a/b");
        fs::create_dir_all(&sub).unwrap();
        let loaded = load_project(&sub).unwrap().unwrap();
        assert_eq!(loaded.manager, Some(Manager::Bun));
    }

    #[test]
    fn no_project_config_returns_none() {
        let dir = tempdir().unwrap();
        assert!(load_project(dir.path()).unwrap().is_none());
    }
}
