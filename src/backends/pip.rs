use super::{Backend, Resolved};
use crate::flags::NormalisedFlags;

pub struct Pip;

impl Backend for Pip {
    fn install(&self, pkgs: &[String], flags: &NormalisedFlags) -> Resolved {
        let mut args = vec!["install".to_string()];
        let mut warnings = Vec::new();
        if flags.global {
            args.push("--user".into());
            warnings.push("pip uses --user for non-system installs; consider a venv".into());
        }
        if flags.dev {
            warnings.push("pip has no native dev-dependency concept; package will be installed normally".into());
        }
        if flags.exact {
            warnings.push("pip does not have an --exact flag; pin via pkg==X.Y.Z in the package spec".into());
        }
        args.extend(pkgs.iter().cloned());
        Resolved {
            program: "pip".into(),
            args,
            warnings,
        }
    }
}
