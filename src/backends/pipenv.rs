use super::{Backend, Resolved};
use crate::flags::NormalisedFlags;

pub struct Pipenv;

impl Backend for Pipenv {
    fn install(&self, pkgs: &[String], flags: &NormalisedFlags) -> Resolved {
        let mut args = vec!["install".to_string()];
        let mut warnings = Vec::new();
        if flags.global {
            warnings.push("pipenv has no global install; ignoring -g".into());
        }
        if flags.dev {
            args.push("--dev".into());
        }
        if flags.exact {
            warnings.push("pipenv has no --exact flag; use pkg==X.Y.Z to pin".into());
        }
        args.extend(pkgs.iter().cloned());
        Resolved {
            program: "pipenv".into(),
            args,
            warnings,
        }
    }
}
