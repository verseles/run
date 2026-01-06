---
feature: "run - Universal Task Runner"
spec: |
  CLI tool in Rust that abstracts command execution by auto-detecting
  the development environment (Node.js, Python, Rust, PHP, Go, Ruby, Java,
  .NET, Elixir, Swift, Zig, Make) and delegating to the appropriate tool.
  Eliminates the need to remember which package manager each project uses.
  
  Repository: https://github.com/verseles/run
  
  Performance targets:
  - Binary size < 5MB
  - Cold start < 50ms
  - Recursive search (3 levels) < 10ms
---

## Task List

### Feature 1: MVP Core - Project Setup & Infrastructure
Description: Establish project foundation with Cargo structure, CI/CD pipeline, and development tooling

- [x] 1.01 Create Cargo.toml with metadata and dependencies - Commit: e562098
- [x] 1.02 Configure release profile for binary optimization (LTO, strip, panic=abort) - Commit: e562098
- [x] 1.03 Set up GitHub Actions CI workflow with lint, test, security, build jobs - Commit: 813c9f4
- [x] 1.04 Add cross-compilation matrix (Linux/macOS/Windows, x86_64/aarch64) - Commit: 813c9f4
- [x] 1.05 Create pre-push.sh script for local CI simulation - Commit: 813c9f4
- [x] 1.06 Add .gitignore with Rust patterns - Commit: 9ee1e99
- [x] 1.07 Include AGPL-3.0 LICENSE file - Commit: e562098

### Feature 2: MVP Core - CLI Interface
Description: Implement command-line argument parsing with all required flags and behaviors

- [x] 2.01 Implement CLI parsing with clap derive (src/cli.rs) - Commit: e562098
- [x] 2.02 Add --levels=N flag (default: 3, min: 0, max: 10) for recursive search depth - Commit: e562098
- [x] 2.03 Add --ignore=tool1,tool2 flag to skip specific runners - Commit: e562098
- [x] 2.04 Add -v/--verbose flag for detailed detection info - Commit: e562098
- [x] 2.05 Add -q/--quiet flag to suppress CLI messages - Commit: e562098
- [x] 2.06 Add --dry-run flag to show command without executing - Commit: e562098
- [x] 2.07 Add --update flag for forced synchronous update - Commit: e562098
- [x] 2.08 Add -h/--help and -V/--version flags - Commit: e562098
- [x] 2.09 Implement -- separator for passing arguments to underlying command - Commit: e562098
- [x] 2.10 Capture and return original exit code from executed command - Commit: e562098
- [x] 2.11 Define specific exit codes (1=generic, 2=runner not found, 3=conflict, 127=tool missing) - Commit: e562098

### Feature 3: MVP Core - Runner Detection
Description: Implement detection logic for 20+ package managers and build tools with priority ordering

- [x] 3.01 Create detector trait/interface for extensible detection (src/detectors/mod.rs) - Commit: e562098
- [x] 3.02 Implement Node.js detector: Bun, PNPM, Yarn, NPM priority order - Commit: e562098
- [x] 3.03 Implement Python detector: UV, Poetry, Pipenv, Pip priority order - Commit: e562098
- [x] 3.04 Implement Rust detector: Cargo - Commit: e562098
- [x] 3.05 Implement PHP detector: Composer - Commit: e562098
- [x] 3.06 Implement Go detector: Taskfile, Go Modules - Commit: e562098
- [x] 3.07 Implement Ruby detector: Bundler, Rake - Commit: e562098
- [x] 3.08 Implement Java detector: Gradle, Maven - Commit: e562098
- [x] 3.09 Implement .NET detector: csproj/sln files - Commit: e562098
- [x] 3.10 Implement Elixir detector: Mix - Commit: e562098
- [x] 3.11 Implement Swift detector: Package.swift - Commit: e562098
- [x] 3.12 Implement Zig detector: build.zig - Commit: e562098
- [x] 3.13 Implement Make detector: Makefile/makefile (case-insensitive) - Commit: b60383b
- [x] 3.14 Implement recursive search up to N levels (default 3) - Commit: e562098
- [x] 3.15 Implement lockfile conflict detection and resolution - Commit: e562098

### Feature 4: MVP Core - Execution Engine
Description: Execute detected commands with proper I/O handling and signal forwarding

- [x] 4.01 Implement process spawning with connected stdin/stdout/stderr (src/runner.rs) - Commit: e562098
- [x] 4.02 Handle command execution on Unix with proper signal forwarding - Commit: e562098
- [x] 4.03 Handle command execution on Windows with CommandExt - Commit: 6d57fc8
- [x] 4.04 Capture and propagate exit codes correctly - Commit: e562098

### Feature 5: MVP Core - Configuration System
Description: TOML-based configuration with global/local precedence

- [x] 5.01 Implement global config parsing (~/.config/run/config.toml) - Commit: e562098
- [x] 5.02 Implement local project config (./run.toml) - Commit: e562098
- [x] 5.03 Implement config precedence: defaults < global < local < CLI args - Commit: e562098
- [x] 5.04 Support config options: max_levels, auto_update, ignore_tools, verbose, quiet - Commit: e562098

### Feature 6: MVP Core - Auto-Update System
Description: Background auto-update via GitHub Releases with notification

- [x] 6.01 Implement GitHub Releases API check (src/update.rs) - Commit: e562098
- [x] 6.02 Implement semver comparison for version detection - Commit: e562098
- [x] 6.03 Implement platform/architecture detection for correct asset download - Commit: e562098
- [x] 6.04 Implement background update daemon (post-command execution) - Commit: e562098
- [x] 6.05 Implement atomic binary replacement with temp file rename - Commit: e562098
- [x] 6.06 Store update metadata in ~/.config/run/update.json - Commit: e562098
- [x] 6.07 Show update notification with changelog on next run - Commit: e562098
- [x] 6.08 Respect RUN_NO_UPDATE env var and auto_update config - Commit: e562098
- [x] 6.09 Implement --update flag for forced synchronous update - Commit: e562098

### Feature 7: MVP Core - User Experience
Description: Colored output, icons, and shell completions

- [x] 7.01 Implement colored output with owo-colors/colored (src/output.rs) - Commit: e562098
- [x] 7.02 Add Unicode icons for status messages - Commit: e562098
- [x] 7.03 Respect NO_COLOR environment variable - Commit: e562098
- [x] 7.04 Generate shell completions with clap_complete (bash, zsh, fish, powershell) - Commit: e562098
- [x] 7.05 Document shell completion installation in README - Commit: 20af6af

### Feature 8: MVP Core - Distribution
Description: Install scripts and release automation

- [x] 8.01 Create install.sh for Linux/macOS with platform detection - Commit: e562098
- [x] 8.02 Create install.ps1 for Windows PowerShell - Commit: 121527e
- [x] 8.03 Include SHA256 checksum verification in install scripts - Commit: e562098
- [x] 8.04 Set up GitHub Actions release workflow for multi-platform builds - Commit: 813c9f4
- [x] 8.05 Create comprehensive README.md with badges, examples, tables - Commit: e562098
- [x] 8.06 Publish v0.1.0 release - Commit: e562098
- [x] 8.07 Publish v0.1.1 with bug fixes - Commit: 217d180

### Feature 9: MVP Polish - Testing & Quality
Description: Comprehensive testing and code quality improvements

- [x] 9.01 Create test fixtures for each ecosystem (tests/fixtures/) - Commit: (pending)
- [x] 9.02 Add unit tests for each detector module (73 unit tests) - Commit: (pending)
- [x] 9.03 Add integration tests using assert_cmd crate (56 tests) - Commit: (pending)
- [x] 9.04 Add tests for config precedence logic - Commit: (pending)
- [x] 9.05 Add tests for CLI argument parsing edge cases - Commit: (pending)
- [x] 9.06 Add tests for recursive search behavior - Commit: (pending)
- [x] 9.07 Add tests for conflict detection scenarios - Commit: (pending)
- [x] 9.08 Set up code coverage reporting with cargo-tarpaulin - Commit: (pending)
- [x] 9.09 Add property-based tests with proptest (16 tests) - Commit: (pending)

### Feature 10: MVP Polish - Documentation & Demos
Description: Visual demos and enhanced documentation

- [ ] 10.01 Create Asciinema/VHS demo showing basic usage
- [ ] 10.02 Create demo for conflict resolution scenario
- [ ] 10.03 Create demo for recursive search from subdirectory
- [ ] 10.04 Add demo GIF/link to README hero section

### Feature 11: v0.2.0+ - Distribution Expansion
Description: Publish to package managers and package registries

- [ ] 11.01 Publish to crates.io as run-cli
- [ ] 11.02 Create Homebrew tap (verseles/tap/run)
- [ ] 11.03 Create Scoop manifest for Windows
- [ ] 11.04 Create Chocolatey package for Windows
- [ ] 11.05 Create AUR PKGBUILD for Arch Linux

### Feature 12: v0.2.0+ - Performance & Caching
Description: Detection cache and performance optimizations

- [!] 12.01 Implement detection result cache - REJECTED: CLI executes once and exits, no session to cache
- [!] 12.02 Add parallelization for multiple directory checks - REJECTED: Max 4 dirs, thread overhead > gain
- [!] 12.03 Profile with cargo flamegraph - REJECTED: Detection is just Path::exists(), already <3ms
- [!] 12.04 Verify binary size stays < 5MB - REJECTED: Already ~2MB, convert to CI check if needed
- [!] 12.05 Verify cold start stays < 50ms - REJECTED: Already ~3ms, convert to CI check if needed

Note: Feature rejected as premature optimization. Current performance is excellent (<3ms detection).
      If performance issues arise in the future, revisit with actual profiling data.

### Feature 13: v0.2.0+ - Workspace & Monorepo Support
Description: Support for monorepo tools and workspace detection

- [ ] 13.01 Detect Nx workspace (nx.json)
- [ ] 13.02 Detect Turborepo workspace (turbo.json)
- [ ] 13.03 Detect Lerna workspace (lerna.json)
- [ ] 13.04 Detect pnpm workspace (pnpm-workspace.yaml)
- [ ] 13.05 Detect npm/yarn workspaces (package.json workspaces field)
- [ ] 13.06 Support Corepack detection via packageManager field

### Feature 14: v0.3.0+ - Telemetry & Analytics
Description: Optional anonymous usage statistics

- [ ] 14.01 Implement opt-in anonymous telemetry system
- [ ] 14.02 Track runner usage frequency (which tools are most used)
- [ ] 14.03 Add config option to disable telemetry
- [ ] 14.04 Create dashboard for aggregated stats

### Feature 15: v1.0.0+ - Extensibility
Description: Plugin system and advanced features

- [ ] 15.01 Design plugin system architecture
- [ ] 15.02 Allow custom runners via .run-plugins/ directory
- [ ] 15.03 Support user-defined aliases (run t -> run test)
- [ ] 15.04 Add pre/post execution hooks
- [ ] 15.05 Add interactive TUI mode for script selection

### Feature 16: v1.0.0+ - IDE Integration
Description: VS Code and other IDE integrations

- [ ] 16.01 Create VS Code extension for run command palette
- [ ] 16.02 Add task provider for VS Code tasks.json
- [ ] 16.03 Document integration with other editors (Neovim, etc.)

### Feature 17: v1.0.0+ - Container Support
Description: Docker/container-aware execution

- [ ] 17.01 Detect Dockerfile presence
- [ ] 17.02 Detect docker-compose.yml/docker-compose.yaml
- [ ] 17.03 Option to execute commands inside container

## Notes

- v0.1.1 released: 217d180 (docs: release v0.1.1)
- Makefile case-sensitivity fix: b60383b
- Windows CommandExt fix: 6d57fc8
