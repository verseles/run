# Best practices

- Read the @README.md file to understand the project.
- Read the @AGENTS.md file to understand the best practices for this project.
- Every time you end a asked task, call the tool funcion "play_notification" to notify the user. Then make a commit. Only push when asked.

## Testing instructions

- Find the CI plan in the .github/workflows folder.
- Run `./scripts/pre-push.sh` to run every check defined for that package.
- From the package root you can just call `./scripts/pre-push.sh`. The commit should pass all tests before you merge.
- Fix any test or type errors until the whole suite is green.
- After moving files or changing imports, run `./scripts/pre-push.sh` to be sure rules still pass.
- Add or update tests for the code you change, even if nobody asked.

## Before PR

- Update CHANGELOG.md with the changes you made.
- If not tagged, add as [Unreleased] section.
- If asked for a tag, move the [Unreleased] section to the new tag.
- Verify carefully if @README.md is updated or need to be updated.
- Based on last session, verify if @AGENTS.md ADR memories need to be updated or added new memories.

## PR instructions

- Title format: [<project_name>] <Title>
- Always run `./scripts/pre-push.sh` before committing.

## ADR Memories

### ADR-001: Crate Selection for CLI

- **clap** with derive feature for CLI parsing (mature, well-documented)
- **tokio** for async runtime (most adopted in Rust ecosystem)
- **reqwest** with rustls-tls for HTTP (avoids OpenSSL dependency issues)
- **owo-colors** for terminal styling (lightweight, supports NO_COLOR convention)
- **thiserror** + **anyhow** combo for error handling (ergonomic errors)
- **toml** for configuration files (human-readable, Rust ecosystem standard)

### ADR-002: Detector Architecture

- Each package manager detector lives in `src/detectors/<ecosystem>.rs`
- Detectors follow priority order: more specific (lockfiles) before generic (manifests)
- Within ecosystems, modern tools prioritized over legacy (e.g., bun > pnpm > yarn > npm)
- Make detector is last fallback (most generic utility)

### ADR-003: Cross-Platform Considerations

- Use `which` crate instead of raw shell commands for tool detection
- Makefile detection uses `read_dir` to get exact filename for case-insensitive filesystems (macOS)
- Windows-specific imports guarded with `#[cfg(windows)]` (e.g., `CommandExt` for `creation_flags`)
- Shell completions generated for bash, zsh, fish, and powershell

### ADR-004: Binary Optimization Profile

- Release profile uses `lto = true`, `strip = true`, `panic = "abort"`, `opt-level = "z"`, `codegen-units = 1`
- Target: binary size < 5MB across all platforms

### ADR-005: Auto-Update Strategy

- Runs asynchronously AFTER command execution completes
- Spawns detached daemon process to avoid blocking user workflow
- Silent failures (network issues, permissions) - never interrupt UX
- Stores update metadata in `~/.config/run/update.json`
- Shows changelog notification on next run (respects --quiet flag)

### ADR-006: Exit Code Semantics

- Pass through original command exit code unchanged (critical for CI/CD)
- CLI-specific errors use distinct codes: 1 (generic), 2 (runner not found), 3 (lockfile conflict), 127 (tool not installed)

### ADR-007: Configuration Precedence

- Order: hardcoded defaults → `~/.config/run/config.toml` (global) → `./run.toml` (local) → CLI args
- Unknown keys silently ignored for forward compatibility

### ADR-008: Shell Completions Installation

- Bash: `~/.local/share/bash-completion/completions/run` (user-local, no sudo)
- Zsh: requires fpath and compinit setup in `~/.zshrc`
- Fish: `~/.config/fish/completions/run.fish` (with mkdir if needed)
- PowerShell: creates `$PROFILE` if it doesn't exist
