# PLANO DE DESENVOLVIMENTO: CLI "run"

## OBJETIVO

Criar uma ferramenta de linha de comando (CLI) em Rust chamada `run` que abstrai a execução de comandos de projeto, detectando automaticamente o ambiente de desenvolvimento (Node.js, Python, Rust, PHP, Go, Ruby, Java, .NET, Elixir, Swift, Zig, Make) e delegando para a ferramenta apropriada, eliminando a necessidade de memorizar qual gerenciador cada projeto utiliza.

**Repositório**: https://github.com/verseles/run

***

## ORIENTAÇÃO CRÍTICA PARA DESENVOLVIMENTO

**ANTES de iniciar qualquer implementação**, realizar pesquisa web para informações atualizadas sobre:
- Convenções atuais de lockfiles para cada ecossistema
- Comandos padrão de execução de cada ferramenta
- Breaking changes recentes em gerenciadores de pacotes
- Melhores práticas de estrutura de projeto Rust para CLIs
- Crates Rust mais recentes e estáveis para: parsing CLI, self-update, async runtime, colorização
- Estrutura de GitHub Releases API e autenticação
- Cross-compilation best practices para Rust

Realizar pesquisas incrementais durante desenvolvimento quando houver dúvidas sobre implementação específica. Não assumir conhecimento desatualizado.

***

## ARQUITETURA DE DETECÇÃO

### Hierarquia de Prioridade Global

Verificar a presença de arquivos-chave na seguinte ordem de precedência:

**Ecossistema Node.js**:
1. **Bun**: `bun.lockb` OU `bun.lock` + `package.json` → `bun run <comando>`
2. **PNPM**: `pnpm-lock.yaml` + `package.json` → `pnpm run <comando>`
3. **Yarn**: `yarn.lock` + `package.json` → `yarn run <comando>`
4. **NPM**: `package-lock.json` + `package.json` OU apenas `package.json` → `npm run <comando>`

**Ecossistema Python**:
5. **UV**: `uv.lock` + `pyproject.toml` → `uv run <comando>`
6. **Poetry**: `poetry.lock` + `pyproject.toml` → `poetry run <comando>`
7. **Pipenv**: `Pipfile.lock` + `Pipfile` → `pipenv run <comando>`
8. **Pip**: `requirements.txt` OU `pyproject.toml` (sem lock de poetry/uv) → `python -m <comando>`

**Ecossistema Rust**:
9. **Cargo**: `Cargo.toml` + `Cargo.lock` → `cargo <comando>`

**Ecossistema PHP**:
10. **Composer**: `composer.lock` + `composer.json` → `composer run <comando>`

**Ecossistema Go**:
11. **Taskfile**: `Taskfile.yml` OU `Taskfile.yaml` → `task <comando>`
12. **Go Modules**: `go.mod` + `go.sum` → `go run <comando>` (se comando parecer caminho) OU `go <comando>`

**Ecossistema Ruby**:
13. **Bundler**: `Gemfile.lock` + `Gemfile` → `bundle exec <comando>`
14. **Rake**: `Rakefile` → `rake <comando>`

**Ecossistema Java/JVM**:
15. **Gradle**: `build.gradle` OU `build.gradle.kts` + `gradle.lockfile` (opcional) → `gradle <comando>`
16. **Maven**: `pom.xml` → `mvn <comando>`

**Ecossistema .NET**:
17. **.NET**: `*.csproj` OU `*.sln` → `dotnet <comando>`

**Ecossistema Elixir**:
18. **Mix**: `mix.exs` + `mix.lock` → `mix <comando>`

**Ecossistema Swift**:
19. **Swift Package Manager**: `Package.swift` → `swift run <comando>`

**Ecossistema Zig**:
20. **Zig Build**: `build.zig` → `zig build <comando>`

**Utilitário Genérico**:
21. **Make**: `Makefile` OU `makefile` → `make <comando>`

**Racionalização da Ordem**:
- Priorizar ferramentas mais específicas antes de genéricas (lockfiles antes de manifestos simples)
- Dentro de cada ecossistema, priorizar ferramentas modernas sobre legado
- Make fica por último por ser mais genérico e usado como fallback universal

### Estratégia de Busca Recursiva

1. Verificar diretório atual (`./`)
2. Se nenhum runner for encontrado, subir um nível (`../`)
3. Repetir até **3 níveis acima** por padrão (configurável via `--levels=N`)
4. Se nada for encontrado após limite, retornar erro formatado:
   ```
   Erro: Nenhum runner encontrado em 3 níveis acima do diretório atual.
   Dica: Use --levels=N para aumentar a busca ou --ignore=<tool> para ajustar detecção.
   ```

Implementar cache inteligente: se o diretório já foi scaneado na mesma execução, reutilizar resultado.

### Resolução de Conflitos de Lockfiles

Quando múltiplos lockfiles do **mesmo ecossistema** forem encontrados (exemplo: `package-lock.json` + `yarn.lock`):

1. Verificar quais ferramentas correspondentes estão instaladas globalmente usando `which` (Unix) ou `where` (Windows)
2. **Se apenas uma ferramenta estiver instalada**: usar essa e emitir aviso colorido (amarelo):
   ```
   ⚠ Aviso: Encontrados package-lock.json e yarn.lock, mas apenas npm está instalado.
   Usando npm. Considere remover yarn.lock se não estiver em uso.
   ```
3. **Se ambas estiverem instaladas**: parar com erro (vermelho):
   ```
   ❌ Erro: Detectados package-lock.json e yarn.lock.
   Ambas ferramentas (npm, yarn) estão instaladas globalmente.
   Ação necessária: Remova o lockfile defasado ou use --ignore=npm (ou --ignore=yarn).
   ```

4. **Se nenhuma estiver instalada**: erro informativo sugerindo instalação

Aplicar lógica similar para outros ecossistemas (Poetry vs UV, Gradle vs Maven quando ambos presentes).

***

## INTERFACE DE LINHA DE COMANDO

### Sintaxe Base
```
run <comando> [argumentos] [flags] [-- argumentos-extras]
```

### Flags Obrigatórias

Implementar as seguintes flags com parsing robusto:

- `--levels=N`: Define quantos níveis acima do diretório atual buscar (padrão: 3, mínimo: 0, máximo: 10)
- `--ignore=tool1,tool2`: Ignora runners específicos na detecção (aceita lista separada por vírgula)
- `--ignore tool1 --ignore tool2`: Sintaxe alternativa, múltiplas flags (ambas sintaxes devem funcionar)
- `-v, --verbose`: Exibe informações detalhadas de detecção, comando executado, arquivos encontrados
- `-q, --quiet`: Suprime todas as mensagens do próprio CLI (avisos, info), mantém apenas output do comando executado e erros críticos
- `--dry-run`: Exibe o comando completo que seria executado sem executar (útil para debug e scripts)
- `--update`: Força verificação e instalação de update imediato, bloqueante (sobrescreve comportamento padrão assíncrono)
- `-h, --help`: Exibe ajuda completa com lista de todos os runners suportados, exemplos de uso
- `-V, --version`: Exibe versão atual do CLI

### Separador de Argumentos

Implementar suporte ao separador `--` padrão Unix:
```
run test -- --coverage --verbose --reporter=json
```

Todo conteúdo após `--` deve ser repassado literalmente ao comando subjacente, sem parsing ou modificação. Preservar espaços, quotes e caracteres especiais.

### Comportamento de Exit Code

Capturar e retornar o **exit code original** do comando executado, sem modificação. Essencial para integração com CI/CD e scripts bash que dependem de `$?`.

Exceção: se o próprio CLI falhar antes da execução (comando não encontrado, erro de parsing), retornar exit codes específicos:
- `1`: Erro genérico
- `2`: Runner não encontrado
- `3`: Conflito de lockfiles
- `127`: Ferramenta detectada não instalada

***

## CONFIGURAÇÃO

### Arquivo Global: `~/.config/run/config.toml`

Criar estrutura de diretório se não existir. Formato TOML:

```toml
max_levels = 5
auto_update = true
ignore_tools = ["npm"]
verbose = false
quiet = false
```

### Arquivo Local: `./run.toml`

Permite override por projeto:

```toml
max_levels = 2
ignore_tools = ["yarn", "pip"]
verbose = true
```

### Precedência de Configuração

Aplicar na ordem (última sobrescreve anterior):
1. Defaults hardcoded
2. `~/.config/run/config.toml` (global)
3. `./run.toml` (local do projeto)
4. Argumentos CLI

Implementar parsing robusto com validação de tipos e valores. Ignorar silenciosamente keys desconhecidas para compatibilidade futura.

***

## AUTO-ATUALIZAÇÃO

### Estratégia de Update Assíncrona

**Timing crítico**: Executar processo de update **após** o comando solicitado terminar, **antes** do exit do processo principal.

**Fluxo de execução**:
1. CLI recebe `run test`
2. Detecta runner apropriado
3. Executa comando imediatamente (stdout/stderr/exit code conectados ao terminal)
4. Comando termina
5. **Antes de fazer exit**, spawn processo filho detached/daemon que:
   - Consulta GitHub Releases API: `GET https://api.github.com/repos/verseles/run/releases/latest`
   - Compara `tag_name` remoto com versão local (semver parsing)
   - Se versão remota > local:
     - Detecta plataforma/arquitetura atual
     - Baixa asset apropriado (ex: `run-linux-x86_64`, `run-macos-aarch64`, `run-windows-x86_64.exe`)
     - Verifica checksum SHA256 do asset
     - Substitui binário existente atomicamente (rename temp → target)
     - Salva metadados de update em `~/.config/run/update.json`:
       ```json
       {
         "updated_at": "2025-12-14T03:00:00Z",
         "from_version": "0.1.0",
         "to_version": "0.2.0",
         "changelog_url": "https://github.com/verseles/run/releases/tag/v0.2.0"
       }
       ```
   - Processo daemon termina silenciosamente
6. CLI principal faz exit com código do comando executado

**Timeout**: Processo de download tem timeout de **5 segundos**. Se exceder, abortar silenciosamente sem afetar UX.

**Falhas**: Qualquer erro no processo de update (rede, permissões, checksum inválido) deve ser silencioso. Não impactar experiência do usuário.

### Notificação de Update Aplicado

Na **próxima execução** após um update bem-sucedido:

1. Verificar existência de `~/.config/run/update.json`
2. Se existir e `updated_at` for recente (< 24h), exibir mensagem colorida (verde):
   ```
   ✓ run foi atualizado: v0.1.0 → v0.2.0
   
   Mudanças principais:
   - Adicionado suporte para Zig e Swift
   - Melhorada detecção de conflitos
   - Corrigido bug no auto-update do Windows
   
   Ver changelog completo: https://github.com/verseles/run/releases/tag/v0.2.0
   ```
3. Extrair changelog: fazer fetch do release via API e usar campo `body` (resumir primeiras 3-5 linhas se muito longo)
4. Deletar `update.json` após exibir (para não repetir)

Mensagem deve respeitar `--quiet` flag (não exibir se quiet ativo).

### Controle de Update

- Auto-update é **padrão** (opt-out via config `auto_update = false`)
- Flag `--update` força check e instalação **síncrona/bloqueante** antes da execução do comando
- Variável de ambiente `RUN_NO_UPDATE=1` desativa temporariamente

### Tecnologia

Usar:
- Runtime async: **Tokio** (mais maduro e adotado)
- HTTP client: **reqwest** com TLS features
- Crate para self-update: pesquisar e avaliar `self_update` crate ou implementação custom baseada em GitHub API
- Parsing semver: crate `semver`

***

## OTIMIZAÇÃO DE BINÁRIO

### Configuração `Cargo.toml`

Adicionar profile de release otimizado:

```toml
[profile.release]
lto = true              # Link-Time Optimization (cross-crate)
strip = true            # Remove símbolos de debug
panic = "abort"         # Remove stack unwinding
opt-level = "z"         # Otimizar para tamanho
codegen-units = 1       # Máximas otimizações (compilação mais lenta)
```

### Meta de Tamanho

Binário final deve ter **< 5MB** para todas as plataformas (x86_64, aarch64).

Após build de release, executar `strip` adicional se necessário. Considerar `upx` compression para distribuição (testar se não causa problemas com antivírus em Windows).

### Performance

- Cold start (tempo até primeira detecção): **< 50ms**
- Busca recursiva de 3 níveis: **< 10ms**
- Execução de comando não deve adicionar overhead perceptível (< 5ms)

Fazer profiling com `cargo flamegraph` durante desenvolvimento para identificar bottlenecks.

***

## QUALIDADE E TESTES

### Estrutura de Testes

Organizar testes em três categorias:

#### 1. Testes Unitários (`#[test]`)

Para cada módulo de detecção implementar:

**Módulo Node.js** (`src/detectors/node.rs`):
- Detectar corretamente cada tipo de lockfile (bun.lockb, pnpm-lock.yaml, yarn.lock, package-lock.json)
- Priorização quando múltiplos lockfiles existem
- Fallback para package.json sem lock
- Parsing de package.json para extrair scripts (se necessário)

**Módulo Python** (`src/detectors/python.rs`):
- Detectar uv.lock, poetry.lock, Pipfile.lock, requirements.txt
- Priorização UV > Poetry > Pipenv > Pip
- Validar comando gerado para cada ferramenta

Replicar estrutura similar para Go, Ruby, Java, .NET, Elixir, Swift, Zig, Make.

**Módulo Config** (`src/config.rs`):
- Parsing de TOML válido e inválido
- Precedência entre global/local/CLI args
- Defaults corretos quando arquivos não existem

**Módulo CLI** (`src/cli.rs`):
- Parsing de argumentos com clap
- Separador `--` funcionando corretamente
- Flags múltiplas (--ignore repetido)

#### 2. Testes de Integração (`tests/`)

Criar fixtures de projetos reais em `tests/fixtures/`:
```
tests/fixtures/
├── node-bun/          # projeto com bun.lockb
├── node-pnpm/         # projeto com pnpm-lock.yaml
├── python-poetry/     # projeto com poetry.lock
├── rust-cargo/        # projeto Rust
├── mixed-lockfiles/   # conflito intencional
├── nested/
│   └── deep/
│       └── project/   # testar busca recursiva
└── ...
```

**Cenários a testar**:
- Execução end-to-end com comando mock em cada tipo de projeto
- Busca recursiva: executar de subdiretório e verificar que encontra runner N níveis acima
- Conflito de lockfiles: verificar erro apropriado
- Flag `--dry-run`: verificar output sem executar
- Flag `--ignore`: verificar que runner é skipado
- Exit codes corretos

Usar `assert_cmd` crate para testar CLI.

#### 3. Testes de Cross-platform

Configurar CI para executar testes em:
- **Linux**: Ubuntu latest (x86_64)
- **macOS**: latest (x86_64 e aarch64 se possível)
- **Windows**: latest (x86_64)

Atenção especial para:
- Path separators (`/` vs `\`)
- Comandos `which` vs `where`
- Line endings (LF vs CRLF)
- Case sensitivity de filesystem
- Permissões de arquivo (executável em Unix)

### Cobertura de Código

**Meta mínima**: 80% de cobertura para lógica core (detecção, config, CLI parsing).

Excluir de cobertura: formatação de output, códigos de erro específicos, módulo de update (difícil de testar).

Usar `cargo-tarpaulin` ou `cargo-llvm-cov` para gerar relatórios. Integrar ao CI.

### Property-Based Testing

Considerar usar `proptest` para:
- Parsing de caminhos de arquivo com caracteres especiais
- Validação de semver em update checker
- Invariantes de busca recursiva (nunca subir mais que max_levels)

***

## CI/CD PIPELINE

### GitHub Actions Workflow

Criar `.github/workflows/ci.yml`:

**Triggers**:
- Push em `main` e `develop`
- Pull requests
- Tags `v*` (para releases)

**Jobs**:

#### Job 1: Lint (`lint`)

Executar em Ubuntu latest:
```yaml
- cargo fmt --check
- cargo clippy --all-targets --all-features -- -D warnings
```

Falhar build se houver warnings de clippy.

#### Job 2: Test (`test`)

Matrix strategy:
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable]
```

Steps:
```yaml
- cargo test --all-features --verbose
- cargo test --release --all-features  # testar otimizações
```

#### Job 3: Security Audit (`security`)

Executar em Ubuntu latest:
```yaml
- cargo install cargo-audit
- cargo audit
```

Falhar se houver vulnerabilidades HIGH ou CRITICAL.

#### Job 4: Build Release (`build`)

**Trigger**: Apenas em tags `v*`

Matrix para múltiplas plataformas:
```yaml
strategy:
  matrix:
    include:
      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
      - os: ubuntu-latest
        target: aarch64-unknown-linux-gnu
      - os: macos-latest
        target: x86_64-apple-darwin
      - os: macos-latest
        target: aarch64-apple-darwin
      - os: windows-latest
        target: x86_64-pc-windows-msvc
```

Steps:
```yaml
- Instalar cross se necessário
- cargo build --release --target $TARGET
- Gerar checksum SHA256 do binário
- Comprimir (tar.gz para Unix, zip para Windows)
- Upload como artifact
```

#### Job 5: Release (`release`)

**Dependência**: Após `build` job completar com sucesso

**Trigger**: Apenas tags `v*`

Steps:
```yaml
- Download todos os artifacts
- Criar GitHub Release usando tag
- Upload todos os binários + checksums
- Gerar e incluir shell completions (bash, zsh, fish, powershell)
```

Usar action `softprops/action-gh-release` ou similar.

### Simulação Local do CI

Criar script `scripts/pre-push.sh`:

```bash
#!/usr/bin/env bash
set -e

echo "🔍 Executando verificações de CI localmente..."

echo "📝 Verificando formatação..."
cargo fmt --check

echo "🔬 Executando Clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "🧪 Executando testes..."
cargo test --all-features

echo "🔒 Verificando vulnerabilidades..."
cargo audit

echo "✅ Todas verificações passaram!"
```

Tornar executável: `chmod +x scripts/pre-push.sh`

**Instruções no README**: Sugerir instalação como git hook:
```bash
ln -s ../../scripts/pre-push.sh .git/hooks/pre-push
```

***

## EXPERIÊNCIA DO USUÁRIO

### Sistema de Cores e Ícones

Usar biblioteca `owo-colors` ou `colored` para output estilizado.

**Paleta de cores**:
- 🟢 Verde (`#00ff00`): Sucesso, detecção bem-sucedida, update aplicado
- 🟡 Amarelo (`#ffff00`): Avisos, conflitos resolvidos automaticamente
- 🔴 Vermelho (`#ff0000`): Erros críticos, falhas
- 🔵 Azul/Cyan (`#00ffff`): Informações em modo `--verbose`
- ⚪ Branco/Cinza: Output neutro

**Ícones Unicode**:
- ✓ (U+2713): Sucesso
- ⚠ (U+26A0): Aviso
- ❌ (U+274C): Erro
- 🔍 (U+1F50D): Detecção em andamento (verbose)
- 📦 (U+1F4E6): Runner detectado (verbose)
- ⬆ (U+2B06): Update disponível/aplicado

**Exemplo de output**:
```
🔍 Buscando runner em ./src/components...
📦 Detectado: pnpm (pnpm-lock.yaml)
✓ Executando: pnpm run test

[output do comando...]

✓ Comando concluído com sucesso (exit code: 0)
```

Respeitar variável de ambiente `NO_COLOR` (convenção Unix) para desabilitar cores.

### Shell Completions

Gerar completions usando `clap_complete`:

**Targets**:
- Bash: `run.bash`
- Zsh: `_run`
- Fish: `run.fish`
- PowerShell: `_run.ps1`

Incluir nos releases. Adicionar instruções no README para instalação:

**Bash**:
```bash
sudo cp run.bash /usr/share/bash-completion/completions/run
```

**Zsh**:
```bash
cp _run ~/.zsh/completion/
```

**Fish**:
```bash
cp run.fish ~/.config/fish/completions/
```

**PowerShell**:
```powershell
# Adicionar ao $PROFILE
```

Completions devem sugerir:
- Flags disponíveis (`--levels`, `--ignore`, etc.)
- Valores para `--ignore` (lista de runners: npm, yarn, pnpm, etc.)
- Scripts do `package.json` do projeto atual (feature avançada, opcional)

***

## DISTRIBUIÇÃO

### Prioridade 1: Install Script

Criar `install.sh` na raiz do repositório:

**Responsabilidades**:
1. Detectar SO e arquitetura automaticamente (`uname -s`, `uname -m`)
2. Mapear para nome de asset correto no GitHub Release
3. Baixar latest release de `https://github.com/verseles/run/releases/latest`
4. Verificar checksum SHA256 (baixar arquivo `.sha256` correspondente)
5. Instalar em diretório apropriado:
   - Preferência: `$HOME/.local/bin` (se existir ou criar)
   - Fallback: `/usr/local/bin` (se tiver permissão sudo)
   - Windows: `%USERPROFILE%\.local\bin` ou `C:\Program Files\run\`
6. Tornar executável (`chmod +x` em Unix)
7. Verificar se diretório está no PATH, avisar se não estiver
8. Se executado novamente: detectar instalação existente e atualizar

**Comportamento em update**:
```bash
curl -fsSL https://raw.githubusercontent.com/verseles/run/main/install.sh | bash
```

Output esperado:
```
🔍 Detectando sistema: Linux x86_64
📦 Baixando run v0.2.0...
✓ Checksum verificado
✓ Instalado em ~/.local/bin/run
⚠ Adicione ~/.local/bin ao seu PATH:
  export PATH="$HOME/.local/bin:$PATH"
```

Criar também `install.ps1` para Windows (PowerShell).

### Roadmap de Distribuição (Fase 2+)

Adicionar suporte para gerenciadores de pacotes após MVP estável:

**Prioridade 2**:
- `cargo install run-cli` (publicar em crates.io)
- Homebrew tap: `brew install verseles/tap/run`

**Prioridade 3**:
- Scoop (Windows): adicionar a bucket
- Chocolatey (Windows): publicar package
- AUR (Arch Linux): criar PKGBUILD

**Prioridade 4**:
- Snap (Ubuntu/Linux): publicar em snapcraft
- Flatpak: publicar em Flathub
- APT repository: para Debian/Ubuntu
- RPM repository: para Fedora/RHEL

***

## DOCUMENTAÇÃO

### README.md

Estrutura obrigatória:

#### 1. Hero Section
```markdown
# 🚀 run

> Universal task runner for modern development

[![CI](https://github.com/verseles/run/workflows/CI/badge.svg)](...)
[![Release](https://img.shields.io/github/v/release/verseles/run)](...)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue)](...)
```

Incluir logo ASCII art ou imagem.

#### 2. Quick Demo

GIF animado ou Asciinema mostrando:
- Executar `run test` em projeto Node.js (detecta pnpm automaticamente)
- Executar `run build` em projeto Python (detecta poetry)
- Executar de subdiretório (busca recursiva)
- Conflito de lockfiles + resolução

Usar ferramenta como `asciinema` ou `vhs` para gravar.

#### 3. Why run?

Listar problemas que resolve:
- Elimina "qual comando eu uso neste projeto?" (npm vs yarn vs pnpm vs bun)
- Funciona em subdiretórios (não precisa cd para raiz)
- Um comando para governar todos (Node, Python, Rust, Go, Ruby, Java, etc.)
- Auto-update automático (sempre na versão mais recente)
- Zero configuração necessária

#### 4. Installation

```bash
curl -fsSL https://raw.githubusercontent.com/verseles/run/main/install.sh | bash
```

Listar métodos alternativos (cargo install, homebrew, etc. conforme disponíveis).

#### 5. Supported Runners

Tabela visual:

| Ecossistema | Detecção | Comando Executado |
|-------------|----------|-------------------|
| Bun | `bun.lockb` + `package.json` | `bun run <cmd>` |
| PNPM | `pnpm-lock.yaml` + `package.json` | `pnpm run <cmd>` |
| ... | ... | ... |

Incluir todos os 20+ runners suportados.

#### 6. Usage Examples

```bash
# Executar script do projeto
run test

# Passar argumentos extras
run build -- --verbose --production

# Executar de subdiretório (busca recursiva automática)
cd src/components
run lint

# Buscar mais níveis acima
run deploy --levels=5

# Ignorar runners específicos
run start --ignore=npm,yarn

# Modo dry-run (ver comando sem executar)
run build --dry-run

# Modo silencioso
run test -q

# Forçar update
run --update
```

#### 7. Configuration

Exemplos de `~/.config/run/config.toml` e `./run.toml` com comentários explicativos.

#### 8. Shell Completions

Instruções passo-a-passo para cada shell.

#### 9. Advanced Features

- Auto-update em background
- Resolução de conflitos
- Busca recursiva inteligente
- Cross-platform

#### 10. Roadmap

Lista de features planejadas:
- [x] MVP com 20+ runners
- [x] Auto-update
- [ ] Telemetria opt-out
- [ ] Cache de detecção
- [ ] Plugin system
- [ ] VS Code extension

#### 11. Contributing

Link para guia de contribuição (criar quando necessário).

#### 12. License

```
Licensed under GNU Affero General Public License v3.0 (AGPL-3.0)
See LICENSE file for details.
```

### Outros Arquivos

**LICENSE**: Incluir texto completo da AGPL-3.0

**CHANGELOG.md**: Manter atualizado com cada release seguindo formato Keep a Changelog

**CONTRIBUTING.md**: Adicionar quando houver interesse externo de contribuição

***

## LICENÇA

**AGPL-3.0** (GNU Affero General Public License v3.0)

Incluir arquivo `LICENSE` na raiz com texto completo da licença.

**Headers em arquivos fonte**: Adicionar header em cada arquivo Rust:
```rust
// Copyright (C) 2025 [Nome do Autor]
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
```

**Implicações**:
- Código deve permanecer open source
- Modificações devem ser compartilhadas sob mesma licença
- Se usado em serviço de rede, código-fonte deve ser disponibilizado
- Permite uso comercial desde que código permaneça aberto

***

## ROADMAP

### Fase 1: MVP (Versão 0.1.0)

**Entregas obrigatórias**:
- ✅ Detecção de 20+ runners (Node/Python/Rust/PHP/Go/Ruby/Java/.NET/Elixir/Swift/Zig/Make)
- ✅ Busca recursiva configurável (padrão 3 níveis)
- ✅ Resolução de conflitos de lockfiles
- ✅ Flags essenciais (--levels, --ignore, -v, -q, --dry-run, --help, --version)
- ✅ Separador de argumentos (--)
- ✅ Auto-atualização via GitHub Releases (assíncrona pós-execução)
- ✅ Notificação de update com changelog
- ✅ Configuração global + local (TOML)
- ✅ CI/CD completo (Linux/macOS/Windows)
- ✅ Script pre-push para validação local
- ✅ Releases automáticos em tags v*
- ✅ Install script (curl-to-bash)
- ✅ Shell completions (bash/zsh/fish/powershell)
- ✅ README moderno com demos visuais
- ✅ Testes unitários + integração (cobertura > 80%)
- ✅ Binário otimizado (< 5MB)
- ✅ Exit codes apropriados
- ✅ Cores e ícones harmoniosos

**Critérios de lançamento**:
- Todos testes passando em 3 plataformas
- Documentação completa
- Pelo menos 5 linguagens testadas manualmente em projetos reais

### Fase 2: Adoção e Polimento (Versão 0.2.0 - 0.5.0)

**Features**:
- Publicação em crates.io (`cargo install run-cli`)
- Homebrew tap oficial
- Scoop/Chocolatey para Windows
- Cache de detecção (evitar re-scan em múltiplas execuções consecutivas)
- Suporte a workspaces/monorepos (Nx, Turborepo, Lerna)
- Detecção de `package.json` → campo `packageManager` (Corepack)
- Estatísticas de uso anônimas (opt-out via config)
- Melhorias de performance (paralelização de checks)
- Suporte a mais arquiteturas (ARM, RISC-V)

**Métricas de sucesso**:
- 100+ stars no GitHub
- 1000+ instalações
- 5+ contribuidores externos

### Fase 3: Extensibilidade (Versão 1.0.0+)

**Features avançadas**:
- Plugin system (usuários podem adicionar runners customizados via `.run-plugins/`)
- Integração com IDEs (VS Code extension)
- Suporte a containers (detectar Dockerfile/docker-compose, executar via docker)
- AI-powered: sugerir comandos quando script não existe
- Telemetria detalhada com dashboard web (opt-in)
- Suporte a aliases customizados (`run t` → `run test`)
- Hooks pré/pós-execução (executar setup antes do comando)
- Modo interativo (TUI para escolher entre múltiplos scripts)

**Critério para 1.0.0**:
- API estável (breaking changes requerem major bump)
- Produção-ready em ambientes corporativos
- 1000+ stars
- 10000+ instalações ativas

***

## MÉTRICAS DE SUCESSO

### Técnicas (Automatizadas)

**Performance**:
- Cold start < 50ms (medido em CI)
- Busca recursiva 3 níveis < 10ms
- Binary size < 5MB todas plataformas
- Zero regressões de performance entre releases

**Qualidade**:
- Cobertura de testes > 80%
- Zero warnings do Clippy
- Todos testes passando em 3 SOs
- Cargo audit sem vulnerabilidades HIGH/CRITICAL

**Confiabilidade**:
- CI verde > 95% do tempo
- Releases sem rollback
- Issues críticos resolvidos < 48h

### Adoção (Rastreadas)

**Curto prazo (3 meses)**:
- 100 stars GitHub
- 500 instalações via install.sh
- 10 issues/discussions criados por usuários
- 3 contribuidores externos

**Médio prazo (6 meses)**:
- 500 stars GitHub
- 5000 instalações
- 1000 execuções diárias (via telemetria opt-in)
- Mencionado em 3+ artigos/tutoriais

**Longo prazo (12 meses)**:
- 1000+ stars
- 20000+ instalações
- Adotado por projeto open source conhecido
- Empacotado em distro Linux mainstream

***

## INSTRUÇÕES FINAIS DE IMPLEMENTAÇÃO

### Antes de Começar

1. Pesquisar na web estruturas de projeto Rust modernas para CLIs (2024-2025)
2. Avaliar crates mais atualizados para cada funcionalidade
3. Revisar convenções de lockfiles recentes (podem ter mudado)
4. Verificar melhores práticas de GitHub Actions para Rust cross-compilation

### Durante Desenvolvimento

- Fazer commits atômicos com mensagens descritivas (Conventional Commits)
- Testar manualmente em pelo menos 2 SOs diferentes antes de PR
- Executar `scripts/pre-push.sh` antes de cada push
- Documentar decisões arquiteturais importantes (ADRs se necessário)
- Manter CHANGELOG.md atualizado

### Ordem de Implementação Sugerida

1. **Setup básico**: Estrutura do projeto Cargo, CI básico, linting
2. **CLI parsing**: Implementar flags com clap, testes de parsing
3. **Detecção core**: Começar com 3-4 runners (npm, pnpm, cargo, make), busca recursiva
4. **Execução**: Spawn processo, conectar I/O, capturar exit code
5. **Configuração**: Parsing TOML, precedência
6. **Expansão de runners**: Adicionar demais linguagens incrementalmente
7. **Conflitos**: Lógica de resolução de múltiplos lockfiles
8. **Auto-update**: Implementar async após comando
9. **Otimização**: Profile release, reduzir tamanho binário
10. **Completions**: Gerar shell completions
11. **Documentação**: README completo, demos visuais
12. **Release**: Workflow de CI para builds multi-plataforma

### Checklist de Entrega MVP

- [ ] Código compila sem warnings
- [ ] Todos testes passando (unitários + integração)
- [ ] Cobertura > 80%
- [ ] CI verde nas 3 plataformas
- [ ] README completo com exemplos
- [ ] LICENSE incluído
- [ ] Install script funcional
- [ ] Shell completions gerados
- [ ] Binários < 5MB
- [ ] Auto-update testado manualmente
- [ ] Pelo menos 5 runners testados em projetos reais
- [ ] Tag v0.1.0 criada
- [ ] Release publicado no GitHub com assets

***

**FIM DO PLANO**

 
