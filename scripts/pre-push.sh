#!/usr/bin/env bash
# Copyright (C) 2025 Verseles
# SPDX-License-Identifier: AGPL-3.0

# Pre-push hook script to run CI checks locally before pushing

set -e

echo "🔍 Running CI checks locally..."
echo ""

# Check formatting
echo "📝 Checking formatting..."
cargo fmt --check
echo "✓ Formatting OK"
echo ""

# Run Clippy
echo "🔬 Running Clippy..."
cargo clippy --all-targets --all-features -- -D warnings
echo "✓ Clippy OK"
echo ""

# Run tests
echo "🧪 Running tests..."
cargo test --all-features
echo "✓ Tests OK"
echo ""

# Security audit (optional - may not be installed)
if command -v cargo-audit &> /dev/null; then
    echo "🔒 Running security audit..."
    cargo audit
    echo "✓ Security audit OK"
    echo ""
else
    echo "⚠ cargo-audit not installed, skipping security audit"
    echo "  Install with: cargo install cargo-audit"
    echo ""
fi

echo "✅ All checks passed!"
echo ""
