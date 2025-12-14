# 🚀 run

> Universal task runner for modern development

[![CI](https://github.com/verseles/run/workflows/CI/badge.svg)](https://github.com/verseles/run/actions)
[![Release](https://img.shields.io/github/v/release/verseles/run)](https://github.com/verseles/run/releases)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue)](LICENSE)

```
 ██████╗ ██╗   ██╗███╗   ██╗
 ██╔══██╗██║   ██║████╗  ██║
 ██████╔╝██║   ██║██╔██╗ ██║
 ██╔══██╗██║   ██║██║╚██╗██║
 ██║  ██║╚██████╔╝██║ ╚████║
 ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝
 Universal Task Runner
```

**run** automatically detects your project's package manager or build tool and executes commands through the appropriate tool. No more remembering if a project uses npm, yarn, pnpm, bun, poetry, cargo, or any other tool!

## ✨ Features

- 🔍 **Auto-detection** - Automatically detects 20+ package managers and build tools
- 🔄 **Recursive search** - Works from any subdirectory
- ⚡ **Fast** - Cold start < 50ms
- 🔧 **Zero config** - Works out of the box
- 🔄 **Auto-update** - Keeps itself up to date in the background
- 🎨 **Beautiful output** - Colored output with clear status messages
- 🐚 **Shell completions** - Bash, Zsh, Fish, and PowerShell support

## 📦 Installation

### Quick Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/verseles/run/main/install.sh | bash
```

### Cargo Install

```bash
cargo install run-cli
```

### Manual Download

Download the latest release from the [Releases page](https://github.com/verseles/run/releases).

## 🎯 Supported Runners

| Ecosystem | Tool | Detection | Command |
|-----------|------|-----------|---------|
| **Node.js** | Bun | `bun.lockb` / `bun.lock` | `bun run <cmd>` |
| | PNPM | `pnpm-lock.yaml` | `pnpm run <cmd>` |
| | Yarn | `yarn.lock` | `yarn run <cmd>` |
| | NPM | `package-lock.json` / `package.json` | `npm run <cmd>` |
| **Python** | UV | `uv.lock` | `uv run <cmd>` |
| | Poetry | `poetry.lock` | `poetry run <cmd>` |
| | Pipenv | `Pipfile.lock` | `pipenv run <cmd>` |
| | Pip | `requirements.txt` | `python -m <cmd>` |
| **Rust** | Cargo | `Cargo.toml` | `cargo <cmd>` |
| **PHP** | Composer | `composer.lock` | `composer run <cmd>` |
| **Go** | Task | `Taskfile.yml` | `task <cmd>` |
| | Go | `go.mod` | `go <cmd>` |
| **Ruby** | Bundler | `Gemfile.lock` | `bundle exec <cmd>` |
| | Rake | `Rakefile` | `rake <cmd>` |
| **Java** | Gradle | `build.gradle` | `gradle <cmd>` |
| | Maven | `pom.xml` | `mvn <cmd>` |
| **.NET** | dotnet | `*.csproj` / `*.sln` | `dotnet <cmd>` |
| **Elixir** | Mix | `mix.exs` | `mix <cmd>` |
| **Swift** | SPM | `Package.swift` | `swift run <cmd>` |
| **Zig** | Zig | `build.zig` | `zig build <cmd>` |
| **Generic** | Make | `Makefile` | `make <cmd>` |

## 📖 Usage

### Basic Commands

```bash
# Run a script from your project
run test

# Run build
run build

# Run any command
run lint
```

### Passing Arguments

Use `--` to pass arguments to the underlying command:

```bash
# Pass arguments to the test command
run test -- --coverage --verbose

# Build with specific flags
run build -- --production
```

### Working from Subdirectories

**run** automatically searches up the directory tree to find your project's configuration:

```bash
cd src/components
run test  # Finds package.json in parent directories
```

### Command Options

```bash
# Search up to 5 levels (default: 3)
run test --levels=5

# Ignore specific runners
run start --ignore=npm,yarn

# Show detailed detection info
run build --verbose

# Suppress all output except errors
run test --quiet

# Show command without executing (dry run)
run deploy --dry-run

# Force update check
run --update
```

## ⚙️ Configuration

### Global Configuration

Create `~/.config/run/config.toml`:

```toml
# Maximum levels to search above current directory
max_levels = 5

# Enable auto-update (default: true)
auto_update = true

# Tools to always ignore
ignore_tools = ["npm"]

# Enable verbose output by default
verbose = false

# Enable quiet mode by default
quiet = false
```

### Project Configuration

Create `run.toml` in your project root:

```toml
# Override global settings for this project
max_levels = 2
ignore_tools = ["yarn"]
```

### Configuration Precedence

1. CLI arguments (highest priority)
2. Project config (`./run.toml`)
3. Global config (`~/.config/run/config.toml`)
4. Built-in defaults

## 🐚 Shell Completions

Generate completions for your shell:

```bash
# Bash
run completions bash > /usr/share/bash-completion/completions/run

# Zsh
run completions zsh > ~/.zsh/completion/_run

# Fish
run completions fish > ~/.config/fish/completions/run.fish

# PowerShell
run completions powershell >> $PROFILE
```

## 🔄 Auto-Update

**run** automatically updates itself in the background after each command. Updates are silent and non-blocking.

- Updates happen after your command finishes
- Failed updates are silently ignored
- Disable with `RUN_NO_UPDATE=1` or `auto_update = false` in config

After an update, you'll see a notification on the next run:

```
⬆ run was updated: v0.1.0 → v0.2.0

Main changes:
- Added support for Zig and Swift
- Improved conflict detection

See full changelog: https://github.com/verseles/run/releases/tag/v0.2.0
```

## 🔧 Conflict Resolution

When multiple lockfiles from the same ecosystem are detected:

1. **Only one tool installed**: Uses that tool with a warning
2. **Multiple tools installed**: Shows an error with suggested actions
3. **No tools installed**: Shows installation instructions

Example conflict message:
```
❌ Detected Node.js with multiple lockfiles (package-lock.json, yarn.lock).
Both tools (npm, yarn) are installed.
Action needed: Remove the outdated lockfile or use --ignore=npm
```

## 📊 Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (or original command exit code) |
| 1 | Generic error |
| 2 | Runner not found |
| 3 | Lockfile conflict |
| 127 | Tool not installed |

## 🛠️ Development

### Building from Source

```bash
git clone https://github.com/verseles/run.git
cd run
cargo build --release
```

### Running Tests

```bash
cargo test --all-features
```

### Pre-push Checks

```bash
./scripts/pre-push.sh
```

Or install as a git hook:

```bash
ln -s ../../scripts/pre-push.sh .git/hooks/pre-push
```

## 🗺️ Roadmap

- [x] MVP with 20+ runners
- [x] Auto-update via GitHub Releases
- [x] Configuration system
- [x] Shell completions
- [ ] Cache detection results
- [ ] Workspace/monorepo support (Nx, Turborepo)
- [ ] Plugin system for custom runners
- [ ] VS Code extension

## 📄 License

This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**.

See [LICENSE](LICENSE) for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

Made with ❤️ by [Verseles](https://github.com/verseles)
