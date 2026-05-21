# add

One `add` command for every package manager.

`add` looks at your project, figures out whether you're in a JS or Python repo, picks the right package manager (npm / pnpm / yarn / bun / pip / uv / poetry / pipenv), and runs the install for you.

```bash
$ add react
→ pnpm add react

$ add -D pytest
→ uv add --dev pytest

$ add --pm bun zod
→ bun add zod
```

## Why

You've got muscle memory for one PM but every project uses a different one. `add` reads the lockfile (or the Corepack `packageManager` field) and dispatches.

## Install

```sh
# macOS / Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/XavierFabregat/add/releases/latest/download/add-cli-installer.sh | sh

# Windows
powershell -c "irm https://github.com/XavierFabregat/add/releases/latest/download/add-cli-installer.ps1 | iex"

# From source
cargo install --git https://github.com/XavierFabregat/add
```

## Usage

```
add <pkg>...           install package(s)
add -D <pkg>           dev dependency
add -g <pkg>           global install
add -E <pkg>           pin to exact version
add --pm <name> <pkg>  force a specific package manager
add --dry-run <pkg>    print the resolved command, don't run it
add --quiet <pkg>      suppress the `→` echo

add which              print which PM would be used here, and why
add init <pm>          write a .addrc.toml pinning this project to <pm>
add config             show path to the global config
```

Flags are normalised across PMs. `-D` becomes `--save-dev` on npm, `-D` on pnpm, `--dev` on yarn/uv, `-d` on bun, `--group dev` on poetry. Unsupported combinations emit a warning rather than silently dropping the flag.

## How detection works

In order of precedence:

1. `--pm <name>` on the command line.
2. `.addrc.toml` in the current directory or any ancestor.
3. `packageManager` field in `package.json` (Corepack-style: `"pnpm@9.1.0"`).
4. A lockfile in the current directory or any ancestor:
   - JS: `bun.lock` / `bun.lockb` / `pnpm-lock.yaml` / `yarn.lock` / `package-lock.json`
   - Python: `uv.lock` / `poetry.lock` / `Pipfile.lock` / `requirements.txt`
5. A project marker (`package.json`, `pyproject.toml`, `Pipfile`, `setup.py`) plus your configured default.
6. Otherwise: error.

`add which` will tell you which one fired.

## Configuration

**Global** — `~/.config/add/config.toml` (or platform equivalent; `add config` prints the path):

```toml
[defaults]
javascript = "pnpm"   # used when a JS project is detected but has no lockfile
python     = "uv"
```

**Per-project** — `.addrc.toml` at the project root, written by `add init <pm>`:

```toml
manager = "bun"
```

## Supported package managers

| Ecosystem | Managers                              |
|-----------|---------------------------------------|
| JS        | npm, pnpm, yarn, bun                  |
| Python    | pip, uv, poetry, pipenv               |

## Status

v0.1 — JS + Python only. System package managers (brew/apt/winget), more languages (cargo/go/bundler/composer), and `remove` / `update` subcommands are out of scope for now.

## License

MIT.
