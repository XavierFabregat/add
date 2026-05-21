use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn touch(p: &Path) {
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(p, "").unwrap();
}

fn add() -> Command {
    Command::cargo_bin("add").unwrap()
}

#[test]
fn pnpm_lockfile_dispatches_to_pnpm() {
    let dir = tempdir().unwrap();
    touch(&dir.path().join("pnpm-lock.yaml"));
    touch(&dir.path().join("package.json"));
    add()
        .current_dir(dir.path())
        .args(["--dry-run", "react"])
        .assert()
        .success()
        .stderr(contains("→ pnpm add react"));
}

#[test]
fn yarn_lockfile_dispatches_to_yarn() {
    let dir = tempdir().unwrap();
    touch(&dir.path().join("yarn.lock"));
    add()
        .current_dir(dir.path())
        .args(["--dry-run", "lodash"])
        .assert()
        .success()
        .stderr(contains("→ yarn add lodash"));
}

#[test]
fn npm_lockfile_dispatches_to_npm() {
    let dir = tempdir().unwrap();
    touch(&dir.path().join("package-lock.json"));
    add()
        .current_dir(dir.path())
        .args(["--dry-run", "axios"])
        .assert()
        .success()
        .stderr(contains("→ npm install axios"));
}

#[test]
fn bun_lockfile_dispatches_to_bun() {
    let dir = tempdir().unwrap();
    touch(&dir.path().join("bun.lockb"));
    add()
        .current_dir(dir.path())
        .args(["--dry-run", "hono"])
        .assert()
        .success()
        .stderr(contains("→ bun add hono"));
}

#[test]
fn dev_flag_normalises_per_backend() {
    let dir = tempdir().unwrap();
    touch(&dir.path().join("pnpm-lock.yaml"));
    add()
        .current_dir(dir.path())
        .args(["-D", "--dry-run", "vitest"])
        .assert()
        .success()
        .stderr(contains("→ pnpm add -D vitest"));

    let dir2 = tempdir().unwrap();
    touch(&dir2.path().join("package-lock.json"));
    add()
        .current_dir(dir2.path())
        .args(["-D", "--dry-run", "jest"])
        .assert()
        .success()
        .stderr(contains("→ npm install --save-dev jest"));
}

#[test]
fn uv_lockfile_dispatches_to_uv() {
    let dir = tempdir().unwrap();
    touch(&dir.path().join("uv.lock"));
    add()
        .current_dir(dir.path())
        .args(["-D", "--dry-run", "pytest"])
        .assert()
        .success()
        .stderr(contains("→ uv add --dev pytest"));
}

#[test]
fn poetry_dev_flag_uses_group() {
    let dir = tempdir().unwrap();
    touch(&dir.path().join("poetry.lock"));
    add()
        .current_dir(dir.path())
        .args(["-D", "--dry-run", "black"])
        .assert()
        .success()
        .stderr(contains("→ poetry add --group dev black"));
}

#[test]
fn pip_warns_on_dev_flag() {
    let dir = tempdir().unwrap();
    touch(&dir.path().join("requirements.txt"));
    add()
        .current_dir(dir.path())
        .args(["-D", "--dry-run", "requests"])
        .assert()
        .success()
        .stderr(contains("warning: pip has no native dev-dependency"))
        .stderr(contains("→ pip install requests"));
}

#[test]
fn package_manager_field_overrides_lockfile() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{ "name":"x", "packageManager":"pnpm@9.1.0" }"#,
    )
    .unwrap();
    touch(&dir.path().join("yarn.lock"));
    add()
        .current_dir(dir.path())
        .args(["--dry-run", "react"])
        .assert()
        .success()
        .stderr(contains("→ pnpm add react"));
}

#[test]
fn package_manager_field_without_lockfile() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{ "packageManager": "bun@1.1.0" }"#,
    )
    .unwrap();
    add()
        .current_dir(dir.path())
        .args(["--dry-run", "hono"])
        .assert()
        .success()
        .stderr(contains("→ bun add hono"));
}

#[test]
fn pm_override_wins_over_detection() {
    let dir = tempdir().unwrap();
    touch(&dir.path().join("pnpm-lock.yaml"));
    add()
        .current_dir(dir.path())
        .args(["--pm", "yarn", "--dry-run", "react"])
        .assert()
        .success()
        .stderr(contains("→ yarn add react"));
}

#[test]
fn addrc_overrides_detection() {
    let dir = tempdir().unwrap();
    touch(&dir.path().join("pnpm-lock.yaml"));
    fs::write(dir.path().join(".addrc.toml"), "manager = \"bun\"\n").unwrap();
    add()
        .current_dir(dir.path())
        .args(["--dry-run", "hono"])
        .assert()
        .success()
        .stderr(contains("→ bun add hono"));
}

#[test]
fn no_project_errors_with_helpful_message() {
    let dir = tempdir().unwrap();
    add()
        .current_dir(dir.path())
        .args(["--dry-run", "anything"])
        .assert()
        .failure()
        .stderr(contains("no project detected"));
}

#[test]
fn which_prints_manager_and_source() {
    let dir = tempdir().unwrap();
    touch(&dir.path().join("yarn.lock"));
    add()
        .current_dir(dir.path())
        .arg("which")
        .assert()
        .success()
        .stdout(contains("yarn"))
        .stderr(contains("yarn.lock"));
}

#[test]
fn init_writes_addrc_and_which_reads_it() {
    let dir = tempdir().unwrap();
    add()
        .current_dir(dir.path())
        .args(["init", "bun"])
        .assert()
        .success();
    assert!(dir.path().join(".addrc.toml").is_file());
    add()
        .current_dir(dir.path())
        .arg("which")
        .assert()
        .success()
        .stdout(contains("bun"));
}

#[test]
fn quiet_suppresses_arrow_line() {
    let dir = tempdir().unwrap();
    touch(&dir.path().join("pnpm-lock.yaml"));
    add()
        .current_dir(dir.path())
        .args(["-q", "--dry-run", "react"])
        .assert()
        .success()
        .stderr(predicates::str::is_empty());
}
