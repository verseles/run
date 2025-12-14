#!/usr/bin/env bash
set -e

echo "🔍 Executando verificações de CI localmente..."

echo "📝 Verificando formatação..."
cargo fmt --check

echo "🔬 Executando Clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "🧪 Executando testes..."
cargo test --all-features

# cargo audit requires installation, checking if it exists
if command -v cargo-audit &> /dev/null; then
    echo "🔒 Verificando vulnerabilidades..."
    cargo audit
else
    echo "⚠️ cargo-audit não instalado. Pulando verificação de segurança."
fi

echo "✅ Todas verificações passaram!"
