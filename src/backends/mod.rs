use crate::flags::{Manager, NormalisedFlags};

mod bun;
mod npm;
mod pip;
mod pipenv;
mod pnpm;
mod poetry;
mod uv;
mod yarn;

#[derive(Debug, Clone)]
pub struct Resolved {
    pub program: String,
    pub args: Vec<String>,
    pub warnings: Vec<String>,
}

impl Resolved {
    pub fn display(&self) -> String {
        let mut out = self.program.clone();
        for a in &self.args {
            out.push(' ');
            if a.contains(char::is_whitespace) {
                out.push('"');
                out.push_str(a);
                out.push('"');
            } else {
                out.push_str(a);
            }
        }
        out
    }
}

pub trait Backend {
    fn install(&self, pkgs: &[String], flags: &NormalisedFlags) -> Resolved;
}

pub fn backend_for(manager: Manager) -> Box<dyn Backend> {
    match manager {
        Manager::Npm => Box::new(npm::Npm),
        Manager::Pnpm => Box::new(pnpm::Pnpm),
        Manager::Yarn => Box::new(yarn::Yarn),
        Manager::Bun => Box::new(bun::Bun),
        Manager::Pip => Box::new(pip::Pip),
        Manager::Uv => Box::new(uv::Uv),
        Manager::Poetry => Box::new(poetry::Poetry),
        Manager::Pipenv => Box::new(pipenv::Pipenv),
    }
}
