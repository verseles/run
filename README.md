# run

**One command. Any project. Zero configuration.**

[![CI](https://github.com/verseles/run/workflows/CI/badge.svg)](https://github.com/verseles/run/actions)
[![Release](https://img.shields.io/github/v/release/verseles/run)](https://github.com/verseles/run/releases)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue)](LICENSE)

```
run test
```

That's it. Whether your project uses npm, yarn, pnpm, bun, cargo, poetry, gradle, or any of 20+ other tools — `run` figures it out.

## Why?

Every project has its own package manager. Every time you switch projects, you ask yourself:

> "Was this npm or yarn? pnpm? Does it have a Makefile?"

**run** eliminates this friction. Just type `run <command>` and it works.

## Install

```bash
# Linux/macOS
curl -fsSL install.cat/verseles/run | bash

# Windows (PowerShell)
irm install.cat/verseles/run | iex

# Or via Cargo
cargo install run-cli
```

## Usage

```bash
run test              # Runs test with detected tool
run build             # Runs build
run lint              # Runs lint
run dev               # Runs dev server

# Pass arguments after --
run test -- --coverage --watch

# Works from any subdirectory
cd src/components && run test    # Finds package.json in parent dirs
```

## Supported Tools

| Ecosystem | Tools (priority order) |
|-----------|----------------------|
| **Node.js** | bun → pnpm → yarn → npm |
| **Python** | uv → poetry → pipenv → pip |
| **Rust** | cargo |
| **PHP** | composer |
| **Go** | task → go |
| **Ruby** | bundler → rake |
| **Java** | gradle → maven |
| **.NET** | dotnet |
| **Elixir** | mix |
| **Swift** | swift (SPM) |
| **Zig** | zig |
| **Generic** | make |

Detection is based on lockfiles first (more specific), then manifest files.

## Options

```bash
run test --dry-run         # Show command without executing
run test --verbose         # Show detection details
run test --quiet           # Suppress output except errors
run test --levels=5        # Search up to 5 parent directories (default: 3)
run test --ignore=npm,yarn # Skip specific runners
run --update               # Force update check
```

## Configuration

Create `~/.config/run/config.toml` for global settings:

```toml
max_levels = 5
auto_update = true
ignore_tools = ["npm"]
```

Or `run.toml` in your project for local overrides.

**Precedence:** CLI args > local config > global config > defaults

## Conflict Resolution

When multiple lockfiles exist (e.g., `package-lock.json` + `yarn.lock`):

1. If only one tool is installed → uses it with a warning
2. If multiple tools installed → error with suggested action
3. If no tools installed → shows installation instructions

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (passes through original exit code) |
| 1 | Generic error |
| 2 | No runner found |
| 3 | Lockfile conflict |
| 127 | Tool not installed |

## Auto-Update

Updates happen silently in the background after commands complete. Disable with `RUN_NO_UPDATE=1` or `auto_update = false`.

## Shell Completions

```bash
# Bash
run completions bash > ~/.local/share/bash-completion/completions/run

# Zsh
run completions zsh > ~/.zsh/completion/_run

# Fish
run completions fish > ~/.config/fish/completions/run.fish

# PowerShell
run completions powershell >> $PROFILE
```

## Development

```bash
git clone https://github.com/verseles/run.git
cd run
make precommit   # Format, lint, test, audit
cargo build --release
```

## License

AGPL-3.0. See [LICENSE](LICENSE).

---

Made with mass production by [Verseles](https://github.com/verseles)
