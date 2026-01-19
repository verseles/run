# CODEBASE.md

> This is the code map of **run** - Universal Task Runner CLI

## Overview

`run` is a Rust CLI tool that automatically detects the project's package manager or build tool and executes commands through the appropriate tool. It supports 20+ runners across 12 ecosystems.

**Key metrics:**
- Binary size target: < 5MB
- Cold start target: < 50ms
- Recursive search (3 levels): < 10ms

## Folder Structure

```
run/
├── src/
│   ├── main.rs           # Entry point, CLI flow orchestration
│   ├── lib.rs            # Library exports, module declarations
│   ├── cli.rs            # Clap-based CLI argument parsing
│   ├── config.rs         # TOML configuration loading (global + local)
│   ├── runner.rs         # Command search, conflict resolution, execution
│   ├── update.rs         # GitHub Releases auto-update system
│   ├── output.rs         # Colored terminal output (owo-colors)
│   ├── error.rs          # Error types and exit codes
│   └── detectors/        # Package manager detection modules
│       ├── mod.rs        # DetectedRunner struct, Ecosystem enum, detect_all()
│       ├── monorepo.rs   # Nx, Turborepo, Lerna (priority 0)
│       ├── node.rs       # Bun, PNPM, Yarn, NPM (priority 1-4) + Corepack
│       ├── python.rs     # UV, Poetry, Pipenv, Pip (priority 5-8)
│       ├── rust.rs       # Cargo (priority 9)
│       ├── php.rs        # Composer (priority 10)
│       ├── just.rs       # Just (priority 10)
│       ├── go.rs         # Task, Go Modules (priority 11-12)
│       ├── ruby.rs       # Bundler, Rake (priority 13-14)
│       ├── java.rs       # Gradle, Maven (priority 15-16)
│       ├── dotnet.rs     # .NET (priority 17)
│       ├── elixir.rs     # Mix (priority 18)
│       ├── swift.rs      # Swift PM (priority 19)
│       ├── zig.rs        # Zig (priority 20)
│       └── make.rs       # Make (priority 21, fallback)
├── tests/
│   └── integration_test.rs  # CLI integration tests with assert_cmd
├── .github/workflows/
│   └── ci.yml            # CI pipeline (lint, test, security, build, release)
├── Cargo.toml            # Dependencies and release profile
├── Makefile              # Local dev commands (precommit, fmt, clippy, test)
├── install.sh            # Linux/macOS install script
├── install.ps1           # Windows PowerShell install script
├── README.md             # User documentation
├── ROADMAP.md            # Development roadmap
├── AGENTS.md             # AI agent instructions
└── LICENSE               # AGPL-3.0
```

## Entry Point

**`src/main.rs`** - Main execution flow:

```
1. Check for internal update flag (--internal-update-check)
2. Parse CLI arguments (Clap)
3. Load configuration (global + local TOML)
4. Merge config with CLI args (CLI has highest precedence)
5. Check for pending update notifications
6. Handle subcommands (completions)
7. Handle --update flag (synchronous update)
8. Search for runners (recursive up to N levels)
9. Check for lockfile conflicts
10. Execute command via detected runner
11. Spawn background update (if enabled)
12. Exit with original command's exit code
```

## Core Modules

### `cli.rs` - CLI Argument Parsing

Uses **clap** with derive macros. Key structures:

| Struct/Enum | Purpose |
|-------------|---------|
| `Cli` | Main CLI args: command, args, levels, ignore, verbose, quiet, dry_run, update |
| `Commands` | Subcommands: `Completions { shell }` |

Flags:
- `--levels=N` (0-10, default 3) - recursive search depth
- `--ignore=tool1,tool2` - skip specific runners
- `-v/--verbose` - detailed detection info
- `-q/--quiet` - suppress CLI output
- `--dry-run` - show command without executing
- `--update` - force synchronous update

### `config.rs` - Configuration System

TOML-based with precedence: **defaults < global < local < CLI**

| Path | Scope |
|------|-------|
| `~/.config/run/config.toml` | Global |
| `./run.toml` | Project |

Config fields:
- `max_levels: u8` - recursive search depth
- `auto_update: bool` - enable auto-update
- `ignore_tools: Vec<String>` - tools to skip
- `verbose: bool` - verbose output
- `quiet: bool` - quiet mode

### `detectors/mod.rs` - Runner Detection

**`DetectedRunner`** struct:
```rust
pub struct DetectedRunner {
    pub name: String,           // e.g., "pnpm", "cargo"
    pub detected_file: String,  // e.g., "pnpm-lock.yaml"
    pub ecosystem: Ecosystem,   // e.g., NodeJs, Rust
    pub priority: u8,           // lower = higher priority
}
```

**`Ecosystem`** enum: NodeJs, Python, Rust, Php, Go, Ruby, Java, DotNet, Elixir, Swift, Zig, Generic

**`detect_all(dir, ignore_list)`** - runs all detectors in priority order, filters ignored tools, sorts by priority.

**`build_command(task, extra_args)`** - builds the actual command to execute based on runner name.

### `runner.rs` - Execution Engine

| Function | Purpose |
|----------|---------|
| `search_runners(start_dir, max_levels, ignore_list, verbose)` | Recursive search up directory tree |
| `check_conflicts(runners, verbose)` | Detect/resolve lockfile conflicts within same ecosystem |
| `execute(runner, task, extra_args, working_dir, dry_run, verbose, quiet)` | Spawn process, inherit I/O |

**Conflict resolution logic:**
1. If only one tool installed → use it with warning
2. If multiple tools installed → error with instructions
3. If none installed → error suggesting installation

### `update.rs` - Auto-Update System

Uses **reqwest** + **tokio** for async HTTP, **semver** for version comparison.

| Function | Purpose |
|----------|---------|
| `spawn_background_update()` | Detached process for post-command update check |
| `perform_update_check()` | Async: fetch GitHub release, compare versions, download, atomic replace |
| `perform_blocking_update(quiet)` | Synchronous update (--update flag) |
| `check_update_notification(quiet)` | Display pending update notification |

**Update flow:**
1. POST command → spawn detached child process
2. Fetch `https://api.github.com/repos/verseles/run/releases/latest`
3. Compare remote vs local semver
4. Download platform-specific binary
5. Atomic replace (Unix: rename, Windows: backup→rename→delete)
6. Save `~/.config/run/update.json` with changelog
7. Next run shows notification, deletes JSON

### `error.rs` - Error Handling

Uses **thiserror** for error derivation.

| Exit Code | Constant | Meaning |
|-----------|----------|---------|
| 0 | `SUCCESS` | Success |
| 1 | `GENERIC_ERROR` | Generic error |
| 2 | `RUNNER_NOT_FOUND` | No runner detected |
| 3 | `LOCKFILE_CONFLICT` | Multiple lockfiles conflict |
| 127 | `TOOL_NOT_INSTALLED` | Required tool not installed |

### `output.rs` - Terminal Output

Uses **owo-colors** for colored output. Respects `NO_COLOR` env var.

| Function | Icon | Color |
|----------|------|-------|
| `success(msg)` | ✓ | Green |
| `warning(msg)` | ⚠ | Yellow |
| `error(msg)` | ❌ | Red |
| `info(msg)` | 🔍 | Cyan |
| `detected(runner, file)` | 📦 | Blue |
| `executing(cmd)` | ✓ | Green |
| `update_notification(from, to, changelog)` | ⬆ | Green/Yellow |

## Data Flow

```
User: run test --verbose

     ┌─────────────┐
     │  main.rs    │
     └──────┬──────┘
            │ Parse CLI
     ┌──────▼──────┐
     │   cli.rs    │  Cli { command: "test", verbose: true, ... }
     └──────┬──────┘
            │ Load config
     ┌──────▼──────┐
     │  config.rs  │  Merge: defaults < global < local < CLI
     └──────┬──────┘
            │ Search runners
     ┌──────▼──────┐
     │  runner.rs  │  search_runners() → recursive up N levels
     └──────┬──────┘
            │ Detect in each dir
     ┌──────▼──────┐
     │ detectors/  │  detect_all() → Vec<DetectedRunner>
     └──────┬──────┘
            │ Check conflicts
     ┌──────▼──────┐
     │  runner.rs  │  check_conflicts() → single runner or error
     └──────┬──────┘
            │ Build & execute command
     ┌──────▼──────┐
     │  runner.rs  │  execute() → spawn process, inherit I/O
     └──────┬──────┘
            │ Spawn background update
     ┌──────▼──────┐
     │  update.rs  │  spawn_background_update() → detached
     └──────┬──────┘
            │
     ┌──────▼──────┐
     │   EXIT      │  Exit with command's exit code
     └─────────────┘
```

## Detector Priority Table

| Priority | Ecosystem | Runner | Detection File |
|----------|-----------|--------|----------------|
| 0 | Node.js | nx | `nx.json` |
| 0 | Node.js | turbo | `turbo.json` |
| 0 | Node.js | lerna | `lerna.json` |
| 1 | Node.js | bun | `bun.lockb` or `bun.lock` |
| 2 | Node.js | pnpm | `pnpm-lock.yaml` |
| 3 | Node.js | yarn | `yarn.lock` |
| 4 | Node.js | npm | `package-lock.json` or `package.json` |
| 5 | Python | uv | `uv.lock` |
| 6 | Python | poetry | `poetry.lock` |
| 7 | Python | pipenv | `Pipfile.lock` |
| 8 | Python | pip | `requirements.txt` or `pyproject.toml` |
| 9 | Rust | cargo | `Cargo.toml` |
| 10 | PHP | composer | `composer.lock` |
| 10 | Generic | just | `justfile` or `Justfile` |
| 11 | Go | task | `Taskfile.yml` or `Taskfile.yaml` |
| 12 | Go | go | `go.mod` |
| 13 | Ruby | bundler | `Gemfile.lock` |
| 14 | Ruby | rake | `Rakefile` |
| 15 | Java | gradle | `build.gradle` or `build.gradle.kts` |
| 16 | Java | maven | `pom.xml` |
| 17 | .NET | dotnet | `*.csproj` or `*.sln` |
| 18 | Elixir | mix | `mix.exs` |
| 19 | Swift | swift | `Package.swift` |
| 20 | Zig | zig | `build.zig` |
| 21 | Generic | make | `Makefile` or `makefile` |

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI parsing with derive |
| `clap_complete` | Shell completions generation |
| `tokio` | Async runtime for HTTP |
| `reqwest` | HTTP client (rustls-tls) |
| `serde` + `serde_json` + `toml` | Serialization |
| `semver` | Version comparison |
| `owo-colors` | Terminal colors |
| `dirs` | Platform config paths |
| `which` | Check if tool is installed |
| `thiserror` | Error derivation |
| `chrono` | Date/time for update tracking |

## Testing

**Unit tests:** Embedded in each module with `#[cfg(test)]`

**Integration tests:** `tests/integration_test.rs` using `assert_cmd` + `predicates`

Key test scenarios:
- Dry-run detection for each runner
- Recursive search from subdirectories
- --ignore flag behavior
- --levels limit enforcement
- Extra args passthrough (`--`)
- Shell completions generation

## CI/CD Pipeline

**`.github/workflows/ci.yml`**

| Job | Trigger | Purpose |
|-----|---------|---------|
| `lint` | Push/PR | `cargo fmt --check` + `cargo clippy` |
| `test` | Push/PR | Tests on Linux, macOS, Windows |
| `security` | Push/PR | `cargo audit` |
| `build` | Tags `v*` | Cross-compile for 5 platforms |
| `release` | Tags `v*` | Create GitHub Release with artifacts |

**Build targets:**
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu` (via cross)
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

## Main Commands

```bash
# Development
make precommit      # fmt + clippy + test + audit
make fmt            # Check formatting
make clippy         # Run linter
make test           # Run tests
make build          # Debug build
make release        # Release build

# Usage
run test            # Detect runner and execute "test"
run build --dry-run # Show command without executing
run lint --verbose  # Show detection details
run test -- --coverage  # Pass extra args to command
run completions bash    # Generate shell completions
run --update        # Force synchronous update
```

## Points of Attention

1. **Lockfile conflicts** - When multiple lockfiles exist in same ecosystem (e.g., `yarn.lock` + `package-lock.json`), resolution depends on which tools are installed.

2. **Case sensitivity** - Makefile detection uses case-insensitive comparison for macOS compatibility.

3. **Windows support** - Uses `CommandExt::creation_flags` for detached update process, different binary rename strategy.

4. **Auto-update safety** - 5s timeout, silent failures, atomic binary replacement.

5. **Exit code preservation** - Always returns the original command's exit code, except for CLI-specific errors.

6. **NO_COLOR support** - Respects the `NO_COLOR` environment variable for accessibility.

7. **RUN_NO_UPDATE=1** - Environment variable to disable auto-update.
