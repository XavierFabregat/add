use super::{Backend, Resolved};
use crate::flags::{Manager, NormalisedFlags};

pub struct Yarn;

impl Backend for Yarn {
    fn manager(&self) -> Manager {
        Manager::Yarn
    }

    fn install(&self, pkgs: &[String], flags: &NormalisedFlags) -> Resolved {
        let mut args = Vec::new();
        if flags.global {
            args.push("global".to_string());
        }
        args.push("add".to_string());
        if flags.dev {
            args.push("--dev".into());
        }
        if flags.exact {
            args.push("--exact".into());
        }
        args.extend(pkgs.iter().cloned());
        Resolved {
            program: "yarn".into(),
            args,
            warnings: vec![],
        }
    }
}
