# 🚀 run

> Universal task runner for modern development

[![CI](https://github.com/verseles/run/workflows/CI/badge.svg)](https://github.com/verseles/run/actions)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue)](LICENSE)

`run` is a CLI tool that automatically detects and executes scripts/commands based on the project environment. No more guessing if you need `npm run`, `yarn`, `pnpm`, `cargo`, `make`, `maven`, `gradle`, or `python`.

## Why run?

*   **Universal**: Works with Node.js, Python, Rust, Go, Ruby, Java, PHP, .NET, Elixir, Swift, Zig, and Make.
*   **Smart**: Detects lockfiles to choose the correct package manager (e.g., `yarn.lock` -> `yarn`, `pnpm-lock.yaml` -> `pnpm`).
*   **Recursive**: Runs commands from subdirectories by searching up the directory tree.
*   **Conflict Resolution**: Intelligent handling of multiple lockfiles.
*   **Auto-Update**: Automatically keeps itself up to date (configurable).

## Installation

### Script (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/verseles/run/main/install.sh | bash
```

### From Source

```bash
cargo install --path .
```

## Usage

```bash
# Execute a script (e.g., "test" in package.json)
run test

# Pass arguments to the command
run build -- --release

# Search 5 levels up for the runner
run start --levels=5

# Ignore specific runners
run lint --ignore=npm

# Dry run (see what would be executed)
run test --dry-run

# Generate shell completions
run --completion bash > ~/.local/share/bash-completion/completions/run
```

## Supported Runners

| Ecosystem | Detection | Command Executed |
| :--- | :--- | :--- |
| **Node.js** | `bun.lockb` | `bun run <cmd>` |
| | `pnpm-lock.yaml` | `pnpm run <cmd>` |
| | `yarn.lock` | `yarn run <cmd>` |
| | `package.json` | `npm run <cmd>` |
| **Python** | `uv.lock` | `uv run <cmd>` |
| | `poetry.lock` | `poetry run <cmd>` |
| | `Pipfile.lock` | `pipenv run <cmd>` |
| | `requirements.txt` | `python -m <cmd>` |
| **Rust** | `Cargo.toml` | `cargo <cmd>` |
| **Go** | `Taskfile.yml` | `task <cmd>` |
| | `go.mod` | `go <cmd>` |
| **Ruby** | `Gemfile.lock` | `bundle exec <cmd>` |
| | `Rakefile` | `rake <cmd>` |
| **Java** | `build.gradle` | `gradle <cmd>` |
| | `pom.xml` | `mvn <cmd>` |
| **PHP** | `composer.lock` | `composer run <cmd>` |
| **.NET** | `*.csproj` | `dotnet <cmd>` |
| **Elixir** | `mix.exs` | `mix <cmd>` |
| **Swift** | `Package.swift` | `swift run <cmd>` |
| **Zig** | `build.zig` | `zig build <cmd>` |
| **Make** | `Makefile` | `make <cmd>` |

## Configuration

You can configure `run` via `~/.config/run/config.toml` (global) or `run.toml` (local).

```toml
max_levels = 5
auto_update = true
ignore_tools = ["npm"]
verbose = false
quiet = false
```

## License

AGPL-3.0
