use super::{Backend, Resolved};
use crate::flags::NormalisedFlags;

pub struct Bun;

impl Backend for Bun {
    fn install(&self, pkgs: &[String], flags: &NormalisedFlags) -> Resolved {
        let mut args = vec!["add".to_string()];
        if flags.global {
            args.push("-g".into());
        }
        if flags.dev {
            args.push("-d".into());
        }
        let mut warnings = Vec::new();
        if flags.exact {
            warnings.push("bun does not support --exact; ignoring".into());
        }
        args.extend(pkgs.iter().cloned());
        Resolved {
            program: "bun".into(),
            args,
            warnings,
        }
    }
}
