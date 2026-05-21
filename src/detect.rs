use crate::flags::Manager;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ecosystem {
    Javascript,
    Python,
}

#[derive(Debug, Clone)]
pub struct Detection {
    pub manager: Option<Manager>,
    pub ecosystem: Option<Ecosystem>,
    pub root: PathBuf,
    pub source: DetectionSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionSource {
    Lockfile(&'static str),
    Marker(&'static str),
    None,
}

const JS_LOCKFILES: &[(&str, Manager)] = &[
    ("bun.lock", Manager::Bun),
    ("bun.lockb", Manager::Bun),
    ("pnpm-lock.yaml", Manager::Pnpm),
    ("yarn.lock", Manager::Yarn),
    ("package-lock.json", Manager::Npm),
];

const PY_LOCKFILES: &[(&str, Manager)] = &[
    ("uv.lock", Manager::Uv),
    ("poetry.lock", Manager::Poetry),
    ("Pipfile.lock", Manager::Pipenv),
];

const PY_MARKERS: &[&str] = &["pyproject.toml", "Pipfile", "requirements.txt", "setup.py"];
const JS_MARKERS: &[&str] = &["package.json"];

pub fn detect(start: &Path) -> Detection {
    let mut current = Some(start);
    while let Some(dir) = current {
        for (name, mgr) in JS_LOCKFILES {
            if dir.join(name).is_file() {
                return Detection {
                    manager: Some(*mgr),
                    ecosystem: Some(Ecosystem::Javascript),
                    root: dir.to_path_buf(),
                    source: DetectionSource::Lockfile(name),
                };
            }
        }
        for (name, mgr) in PY_LOCKFILES {
            if dir.join(name).is_file() {
                return Detection {
                    manager: Some(*mgr),
                    ecosystem: Some(Ecosystem::Python),
                    root: dir.to_path_buf(),
                    source: DetectionSource::Lockfile(name),
                };
            }
        }
        for marker in JS_MARKERS {
            if dir.join(marker).is_file() {
                return Detection {
                    manager: None,
                    ecosystem: Some(Ecosystem::Javascript),
                    root: dir.to_path_buf(),
                    source: DetectionSource::Marker(marker),
                };
            }
        }
        for marker in PY_MARKERS {
            if dir.join(marker).is_file() {
                let mgr = if *marker == "requirements.txt" {
                    Some(Manager::Pip)
                } else {
                    None
                };
                return Detection {
                    manager: mgr,
                    ecosystem: Some(Ecosystem::Python),
                    root: dir.to_path_buf(),
                    source: DetectionSource::Marker(marker),
                };
            }
        }
        current = dir.parent();
    }
    Detection {
        manager: None,
        ecosystem: None,
        root: start.to_path_buf(),
        source: DetectionSource::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn touch(p: &Path) {
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, "").unwrap();
    }

    #[test]
    fn detects_pnpm_lockfile() {
        let dir = tempdir().unwrap();
        touch(&dir.path().join("pnpm-lock.yaml"));
        touch(&dir.path().join("package.json"));
        let d = detect(dir.path());
        assert_eq!(d.manager, Some(Manager::Pnpm));
        assert_eq!(d.ecosystem, Some(Ecosystem::Javascript));
    }

    #[test]
    fn lockfile_wins_over_marker_at_same_level() {
        let dir = tempdir().unwrap();
        touch(&dir.path().join("yarn.lock"));
        touch(&dir.path().join("package.json"));
        let d = detect(dir.path());
        assert_eq!(d.manager, Some(Manager::Yarn));
    }

    #[test]
    fn walks_up_to_ancestor() {
        let dir = tempdir().unwrap();
        touch(&dir.path().join("uv.lock"));
        let sub = dir.path().join("a/b/c");
        fs::create_dir_all(&sub).unwrap();
        let d = detect(&sub);
        assert_eq!(d.manager, Some(Manager::Uv));
        assert_eq!(d.ecosystem, Some(Ecosystem::Python));
    }

    #[test]
    fn requirements_txt_means_pip() {
        let dir = tempdir().unwrap();
        touch(&dir.path().join("requirements.txt"));
        let d = detect(dir.path());
        assert_eq!(d.manager, Some(Manager::Pip));
    }

    #[test]
    fn bare_pyproject_has_no_manager() {
        let dir = tempdir().unwrap();
        touch(&dir.path().join("pyproject.toml"));
        let d = detect(dir.path());
        assert_eq!(d.manager, None);
        assert_eq!(d.ecosystem, Some(Ecosystem::Python));
    }

    #[test]
    fn detects_bun_text_lockfile() {
        let dir = tempdir().unwrap();
        touch(&dir.path().join("bun.lock"));
        touch(&dir.path().join("package.json"));
        let d = detect(dir.path());
        assert_eq!(d.manager, Some(Manager::Bun));
        assert_eq!(d.source, DetectionSource::Lockfile("bun.lock"));
    }

    #[test]
    fn empty_dir_detects_nothing() {
        let dir = tempdir().unwrap();
        let d = detect(dir.path());
        assert_eq!(d.manager, None);
        assert_eq!(d.ecosystem, None);
    }
}
