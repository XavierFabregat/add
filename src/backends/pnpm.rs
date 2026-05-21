use super::{Backend, Resolved};
use crate::flags::{Manager, NormalisedFlags};

pub struct Pnpm;

impl Backend for Pnpm {
    fn manager(&self) -> Manager {
        Manager::Pnpm
    }

    fn install(&self, pkgs: &[String], flags: &NormalisedFlags) -> Resolved {
        let mut args = vec!["add".to_string()];
        if flags.global {
            args.push("-g".into());
        }
        if flags.dev {
            args.push("-D".into());
        }
        if flags.exact {
            args.push("-E".into());
        }
        args.extend(pkgs.iter().cloned());
        Resolved {
            program: "pnpm".into(),
            args,
            warnings: vec![],
        }
    }
}
