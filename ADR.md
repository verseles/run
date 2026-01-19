# Architecture Decision Records

> Decisões arquiteturais importantes que impactam todo o projeto.

### ADR-001: Crate Selection for CLI

**Status**: ✅ Accepted
**Context**: Selection of core libraries for the CLI tool.
**Decision**: 
- **clap** with derive feature for CLI parsing (mature, well-documented)
- **tokio** for async runtime (most adopted in Rust ecosystem)
- **reqwest** with rustls-tls for HTTP (avoids OpenSSL dependency issues)
- **owo-colors** for terminal styling (lightweight, supports NO_COLOR convention)
- **thiserror** + **anyhow** combo for error handling (ergonomic errors)
- **toml** for configuration files (human-readable, Rust ecosystem standard)
**Consequences**: Standardized stack for Rust CLIs, ensuring performance and maintainability.

### ADR-002: Detector Architecture

**Status**: ✅ Accepted
**Context**: How to structure the detection of package managers.
**Decision**: 
- Each package manager detector lives in `src/detectors/<ecosystem>.rs`
- Detectors follow priority order: more specific (lockfiles) before generic (manifests)
- Within ecosystems, modern tools prioritized over legacy (e.g., bun > pnpm > yarn > npm)
- Make detector is last fallback (most generic utility)
**Consequences**: Modular and extensible architecture for adding new runners.

### ADR-003: Cross-Platform Considerations

**Status**: ✅ Accepted
**Context**: Handling differences between OS platforms.
**Decision**: 
- Use `which` crate instead of raw shell commands for tool detection
- Makefile detection uses `read_dir` to get exact filename for case-insensitive filesystems (macOS)
- Windows-specific imports guarded with `#[cfg(windows)]` (e.g., `CommandExt` for `creation_flags`)
- Shell completions generated for bash, zsh, fish, and powershell
**Consequences**: Robust cross-platform support.

### ADR-004: Binary Optimization Profile

**Status**: ✅ Accepted
**Context**: Reducing binary size for distribution.
**Decision**: 
- Release profile uses `lto = true`, `strip = true`, `panic = "abort"`, `opt-level = "z"`, `codegen-units = 1`
- Target: binary size < 5MB across all platforms
**Consequences**: Small, fast binaries that are easy to download and install.

### ADR-005: Auto-Update Strategy

**Status**: ✅ Accepted
**Context**: Keeping the tool up to date without impacting user experience.
**Decision**: 
- Runs asynchronously AFTER command execution completes
- Spawns detached daemon process to avoid blocking user workflow
- Silent failures (network issues, permissions) - never interrupt UX
- Stores update metadata in `~/.config/run/update.json`
- Shows changelog notification on next run (respects --quiet flag)
**Consequences**: Seamless updates for users.

### ADR-006: Exit Code Semantics

**Status**: ✅ Accepted
**Context**: How to handle exit codes from subcommands.
**Decision**: 
- Pass through original command exit code unchanged (critical for CI/CD)
- CLI-specific errors use distinct codes: 1 (generic), 2 (runner not found), 3 (lockfile conflict), 127 (tool not installed)
**Consequences**: Reliable integration with existing scripts and CI/CD pipelines.

### ADR-007: Configuration Precedence

**Status**: ✅ Accepted
**Context**: Resolving conflicting configuration sources.
**Decision**: 
- Order: hardcoded defaults → `~/.config/run/config.toml` (global) → `./run.toml` (local) → CLI args
- Unknown keys silently ignored for forward compatibility
**Consequences**: Flexible and predictable configuration.

### ADR-008: Shell Completions Installation

**Status**: ✅ Accepted
**Context**: Installing shell completions for different shells.
**Decision**: 
- Bash: `~/.local/share/bash-completion/completions/run` (user-local, no sudo)
- Zsh: requires fpath and compinit setup in `~/.zshrc`
- Fish: `~/.config/fish/completions/run.fish` (with mkdir if needed)
- PowerShell: creates `$PROFILE` if it doesn't exist
**Consequences**: Better user experience with tab completions.

### ADR-009: Testing Strategy

**Status**: ✅ Accepted
**Context**: Choosing testing approach for a CLI tool with multiple ecosystems.
**Decision**: 
- **Unit tests** inline in each module (`#[cfg(test)] mod tests`)
- **Integration tests** using `assert_cmd` + `predicates` for CLI behavior
- **Property-based tests** using `proptest` for invariants (semver, path detection, case-insensitivity)
- **Fixtures** in `tests/fixtures/` for each ecosystem (real lockfiles/manifests)
- **tempfile** for isolated test environments
- All tests run via `cargo test` and `make precommit`
**Consequences**: Comprehensive coverage (145+ tests) with fast feedback. Property tests catch edge cases that example-based tests miss.

### ADR-010: Smart Command Validation with Fallback

**Status**: ✅ Accepted
**Context**: When multiple runners are detected (e.g., Cargo.toml + Makefile), the tool should intelligently select the runner that actually supports the requested command.
**Decision**: 
- Before selecting a runner, validate if it supports the requested command
- For **npm/yarn/pnpm/bun**: Parse `package.json` and check if script exists
- For **cargo**: Check against list of built-in subcommands (build, test, clippy, etc.)
- For **make**: Parse Makefile and extract target names
- For **composer**: Parse `composer.json` scripts section
- For **gradle**: Check built-in tasks + parse `build.gradle` for custom tasks
- For **dotnet**: Check against built-in commands
- Commands return `Supported`, `NotSupported`, or `Unknown` status
- Selection priority: `Supported` > `Unknown` > skip `NotSupported`
- Fallback to first `Unknown` if none explicitly support the command
**Example**: `run precommit` in a Rust project with Makefile correctly runs `make precommit` instead of failing with `cargo precommit`.
**Consequences**: Smarter runner selection that matches user intent. Eliminates need for `--ignore` flag in common fallback scenarios.

### ADR-011: Trait-based Command Validation

**Status**: ✅ Accepted
**Context**: The command validation logic in `src/validators.rs` was centralized in a large match statement. This made it hard to maintain and scale as more ecosystems were added.
**Decision**: 
- Implement a `CommandValidator` trait in `src/detectors/mod.rs`.
- Each detector (e.g., `NodeDetector`, `PythonDetector`) implements this trait.
- Move manifest-specific parsing logic (JSON, YAML, TOML) into the respective detectors.
- `DetectedRunner` holds an `Arc<dyn CommandValidator>` for polymorphic dispatch.
- The old `validators.rs` module was deleted; logic now lives in detector modules.
**Consequences**: Better separation of concerns. Adding support for a new tool's command discovery only requires changing the relevant detector file. Implemented in Feature 18.

### ADR-012: Monorepo Orchestration Tools

**Status**: ✅ Accepted
**Context**: Projects using Nx, Turborepo, or Lerna should use these tools for task execution instead of falling back to raw npm/pnpm/yarn.
**Decision**: 
- Detect `nx.json`, `turbo.json`, `lerna.json` as first-class runners
- Assign priority 0 (highest) - these tools orchestrate underlying package managers
- Build commands: `nx <task>`, `turbo run <task>`, `lerna run <task>`
- Use `UnknownValidator` (trust the tool) - these tools handle command validation internally
**Example**: `run build` in a project with `turbo.json` executes `turbo run build` instead of `npm run build`.
**Consequences**: Proper support for monorepo workflows. Users don't need to remember which orchestration tool their project uses.

### ADR-013: Corepack Package Manager Resolution

**Status**: ✅ Accepted
**Context**: When multiple Node.js lockfiles exist (e.g., `yarn.lock` + `package-lock.json`), the tool needs a way to resolve the conflict automatically.
**Decision**: 
- Read `packageManager` field from `package.json` (Corepack standard)
- Parse format: `"pnpm@9.0.0"` or `"yarn@4.0.0+sha256.abc123"`
- Use this as the primary disambiguation source in `check_conflicts()`
- If `packageManager` specifies a tool that matches a detected lockfile, use it
- Falls back to existing resolution logic if no match or field is missing
**Example**: Project with `yarn.lock`, `package-lock.json`, and `"packageManager": "pnpm@9.0.0"` - the conflict is not resolved by Corepack since none of the lockfiles match pnpm; falls back to installed tools check.
**Consequences**: Respects project's declared package manager intent. Reduces friction in projects migrating between package managers.

### ADR-014: Smart Update System with Throttling

**Status**: Accepted
**Context**: The original auto-update system checked for updates on every CLI execution, which was too aggressive and could cause unnecessary network requests.
**Decision**: 
- Implement a throttled update system with configurable interval (default: 2 hours)
- Store last check timestamp in `~/.config/run/last_update_check`
- Use **hickory-resolver** with Cloudflare DNS (1.1.1.1) for Termux compatibility
- Add `[update]` section to config.toml for fine-grained control:
  ```toml
  [update]
  enabled = true              # Enable auto-update
  check_interval_hours = 2    # Hours between update checks
  ```
- The legacy `auto_update = false` still works but `[update].enabled` takes precedence
**Consequences**: Reduced network overhead, better UX for frequent CLI users, and improved compatibility with restricted environments like Termux.
