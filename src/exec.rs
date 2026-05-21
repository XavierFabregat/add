use crate::backends::Resolved;
use anyhow::{Context, Result};
use std::process::Command;

pub struct RunOptions {
    pub dry_run: bool,
    pub quiet: bool,
}

pub fn run(resolved: &Resolved, opts: RunOptions) -> Result<i32> {
    if !opts.quiet {
        for w in &resolved.warnings {
            eprintln!("warning: {w}");
        }
        eprintln!("→ {}", resolved.display());
    }
    if opts.dry_run {
        return Ok(0);
    }
    let status = Command::new(&resolved.program)
        .args(&resolved.args)
        .status()
        .with_context(|| {
            format!(
                "failed to spawn `{}` — is it installed and on PATH?",
                resolved.program
            )
        })?;
    Ok(status.code().unwrap_or(1))
}
