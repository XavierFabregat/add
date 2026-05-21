mod backends;
mod config;
mod detect;
mod exec;
mod flags;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};
use std::env;

use crate::backends::backend_for;
use crate::config::{GlobalConfig, ProjectConfig};
use crate::detect::{Detection, DetectionSource, Ecosystem, detect};
use crate::exec::{RunOptions, run};
use crate::flags::{Manager, NormalisedFlags};

#[derive(Parser, Debug)]
#[command(
    name = "add",
    version,
    about = "Context-aware package manager dispatcher",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(short = 'D', long = "dev", global = true, help = "Add as a dev dependency")]
    dev: bool,
    #[arg(short = 'g', long = "global", global = true, help = "Install globally")]
    global: bool,
    #[arg(short = 'E', long = "exact", global = true, help = "Pin to exact version")]
    exact: bool,
    #[arg(long = "pm", global = true, value_name = "MANAGER", help = "Force a specific package manager")]
    pm: Option<String>,
    #[arg(long = "dry-run", global = true, help = "Print resolved command without running it")]
    dry_run: bool,
    #[arg(short = 'q', long = "quiet", global = true, help = "Suppress the printed command")]
    quiet: bool,

    #[arg(value_name = "PACKAGE", trailing_var_arg = true, help = "Packages to add (when no subcommand is given)")]
    packages: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print which package manager would be used in the current directory.
    Which,
    /// Write a .addrc.toml in the current directory pinning a manager.
    Init {
        #[arg(value_name = "MANAGER", help = "e.g. pnpm, bun, uv, poetry")]
        manager: String,
    },
    /// Print the path to (and content of) the global config file.
    Config,
}

fn main() {
    let exit = match real_main() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("add: {e:#}");
            1
        }
    };
    std::process::exit(exit);
}

fn real_main() -> Result<i32> {
    let cli = Cli::parse();
    let cwd = env::current_dir().context("reading current directory")?;
    let global = config::load_global().unwrap_or_default();
    let project = config::load_project(&cwd)?;

    match &cli.command {
        Some(Command::Which) => cmd_which(&cwd, &cli, &global, project.as_ref()),
        Some(Command::Init { manager }) => cmd_init(&cwd, manager),
        Some(Command::Config) => cmd_config(),
        None => cmd_add(&cwd, &cli, &global, project.as_ref()),
    }
}

fn cmd_add(
    cwd: &std::path::Path,
    cli: &Cli,
    global: &GlobalConfig,
    project: Option<&ProjectConfig>,
) -> Result<i32> {
    if cli.packages.is_empty() {
        bail!("no packages given; usage: add <package>...");
    }
    let detection = detect(cwd);
    let manager = resolve_manager(cli, &detection, global, project)?;
    let flags = NormalisedFlags {
        dev: cli.dev,
        global: cli.global,
        exact: cli.exact,
    };
    let backend = backend_for(manager);
    let resolved = backend.install(&cli.packages, &flags);
    run(
        &resolved,
        RunOptions {
            dry_run: cli.dry_run,
            quiet: cli.quiet,
        },
    )
}

fn cmd_which(
    cwd: &std::path::Path,
    cli: &Cli,
    global: &GlobalConfig,
    project: Option<&ProjectConfig>,
) -> Result<i32> {
    let detection = detect(cwd);
    match resolve_manager(cli, &detection, global, project) {
        Ok(mgr) => {
            println!("{mgr}");
            if !cli.quiet {
                describe_source(&detection, project, cli);
            }
            Ok(0)
        }
        Err(e) => {
            eprintln!("add: {e:#}");
            Ok(1)
        }
    }
}

fn describe_source(detection: &Detection, project: Option<&ProjectConfig>, cli: &Cli) {
    let reason = if cli.pm.is_some() {
        "--pm override".to_string()
    } else if project.and_then(|p| p.manager).is_some() {
        format!(".addrc.toml at {}", detection.root.display())
    } else {
        match &detection.source {
            DetectionSource::Lockfile(name) => {
                format!("{} at {}", name, detection.root.display())
            }
            DetectionSource::Marker(name) => {
                format!("{} at {} (no lockfile; using configured default)", name, detection.root.display())
            }
            DetectionSource::None => "no project detected".into(),
        }
    };
    eprintln!("  source: {reason}");
}

fn cmd_init(cwd: &std::path::Path, manager: &str) -> Result<i32> {
    let mgr = Manager::parse(manager)
        .ok_or_else(|| anyhow!("unknown manager `{manager}`; expected one of: npm, pnpm, yarn, bun, pip, uv, poetry, pipenv"))?;
    let path = config::write_project_config(cwd, &ProjectConfig { manager: Some(mgr) })?;
    println!("wrote {} (manager = {mgr})", path.display());
    Ok(0)
}

fn cmd_config() -> Result<i32> {
    match config::global_config_path() {
        Some(p) => {
            println!("{}", p.display());
            if p.is_file() {
                let body = std::fs::read_to_string(&p)?;
                print!("---\n{body}");
            } else {
                println!("(file does not exist yet — create it to set defaults)");
            }
            Ok(0)
        }
        None => bail!("could not determine config directory on this platform"),
    }
}

fn resolve_manager(
    cli: &Cli,
    detection: &Detection,
    global: &GlobalConfig,
    project: Option<&ProjectConfig>,
) -> Result<Manager> {
    if let Some(name) = &cli.pm {
        return Manager::parse(name)
            .ok_or_else(|| anyhow!("unknown manager `{name}`"));
    }
    if let Some(m) = project.and_then(|p| p.manager) {
        return Ok(m);
    }
    if let Some(m) = detection.manager {
        return Ok(m);
    }
    match detection.ecosystem {
        Some(Ecosystem::Javascript) => global
            .defaults
            .javascript
            .ok_or_else(|| missing_default("JavaScript", "javascript")),
        Some(Ecosystem::Python) => global
            .defaults
            .python
            .ok_or_else(|| missing_default("Python", "python")),
        None => Err(anyhow!(
            "no project detected in {} (or any parent). Pass --pm <manager> or run `add init <manager>` here.",
            cli_cwd_display()
        )),
    }
}

fn cli_cwd_display() -> String {
    env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "cwd".into())
}

fn missing_default(ecosystem: &str, key: &str) -> anyhow::Error {
    let cfg = config::global_config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "~/.config/add/config.toml".into());
    anyhow!(
        "{ecosystem} project detected but no lockfile present and no default configured.\n\
         Set [defaults]\n{key} = \"...\" in {cfg}, or pass --pm <manager>."
    )
}

