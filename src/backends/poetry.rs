use super::{Backend, Resolved};
use crate::flags::{Manager, NormalisedFlags};

pub struct Poetry;

impl Backend for Poetry {
    fn manager(&self) -> Manager {
        Manager::Poetry
    }

    fn install(&self, pkgs: &[String], flags: &NormalisedFlags) -> Resolved {
        let mut args = vec!["add".to_string()];
        let mut warnings = Vec::new();
        if flags.global {
            warnings.push("poetry has no global add; use `poetry self add` or pipx".into());
        }
        if flags.dev {
            args.push("--group".into());
            args.push("dev".into());
        }
        if flags.exact {
            warnings.push("poetry has no --exact flag; use pkg==X.Y.Z to pin".into());
        }
        args.extend(pkgs.iter().cloned());
        Resolved {
            program: "poetry".into(),
            args,
            warnings,
        }
    }
}
