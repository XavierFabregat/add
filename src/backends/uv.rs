use super::{Backend, Resolved};
use crate::flags::NormalisedFlags;

pub struct Uv;

impl Backend for Uv {
    fn install(&self, pkgs: &[String], flags: &NormalisedFlags) -> Resolved {
        let mut warnings = Vec::new();
        let mut args = Vec::new();
        if flags.global {
            args.push("tool".into());
            args.push("install".into());
            if flags.dev {
                warnings.push("uv tool install ignores --dev".into());
            }
        } else {
            args.push("add".into());
            if flags.dev {
                args.push("--dev".into());
            }
        }
        if flags.exact {
            warnings.push("uv has no --exact flag; pin via pkg==X.Y.Z in the package spec".into());
        }
        args.extend(pkgs.iter().cloned());
        Resolved {
            program: "uv".into(),
            args,
            warnings,
        }
    }
}
