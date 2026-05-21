use super::{Backend, Resolved};
use crate::flags::NormalisedFlags;

pub struct Npm;

impl Backend for Npm {
    fn install(&self, pkgs: &[String], flags: &NormalisedFlags) -> Resolved {
        let mut args = vec!["install".to_string()];
        if flags.global {
            args.push("-g".into());
        }
        if flags.dev {
            args.push("--save-dev".into());
        }
        if flags.exact {
            args.push("--save-exact".into());
        }
        args.extend(pkgs.iter().cloned());
        Resolved {
            program: "npm".into(),
            args,
            warnings: vec![],
        }
    }
}
