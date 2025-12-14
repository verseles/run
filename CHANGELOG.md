# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-12-14

### Added

- Initial release
- Support for 20+ package managers and build tools:
  - Node.js: Bun, PNPM, Yarn, NPM
  - Python: UV, Poetry, Pipenv, Pip
  - Rust: Cargo
  - PHP: Composer
  - Go: Task, Go modules
  - Ruby: Bundler, Rake
  - Java: Gradle, Maven
  - .NET: dotnet
  - Elixir: Mix
  - Swift: Swift Package Manager
  - Zig: Zig build
  - Generic: Make
- Recursive directory search (configurable levels)
- Lockfile conflict detection and resolution
- Auto-update via GitHub Releases
- Global and project-level configuration (TOML)
- Shell completions (Bash, Zsh, Fish, PowerShell)
- Colored terminal output with Unicode icons
- Dry-run mode
- Verbose and quiet modes
- Cross-platform support (Linux, macOS, Windows)

[Unreleased]: https://github.com/verseles/run/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/verseles/run/releases/tag/v0.1.0
