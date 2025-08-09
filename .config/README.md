# 📁 Organização de Arquivos de Configuração

Este documento explica a estrutura de configuração do projeto Toucan e quais arquivos podem ser organizados em pastas.

## 🗂️ Estrutura Atual

```
toucan/
├── .config/                    # 📋 Configurações centralizadas
│   ├── clippy.toml            # Configuração do Clippy (linter)
│   └── workspace-lints.toml   # Lints compartilhados (exemplo)
├── .editorconfig              # 📝 Configuração do editor
├── .gitignore                 # 🚫 Arquivos ignorados pelo Git
├── .vscode/                   # 🏠 Configurações do VS Code
│   ├── extensions.json        # Extensões recomendadas
│   └── settings.json          # Configurações do workspace
├── .github/                   # 🤖 GitHub Actions e workflows
│   ├── dependabot.yml         # Atualizações automáticas
│   └── workflows/ci.yml       # Pipeline CI/CD
├── rustfmt.toml              # 🎨 Formatação de código (raiz obrigatória)
├── Cargo.toml                # 📦 Configuração principal do workspace
└── scripts/                  # 🔧 Scripts utilitários
    └── format.sh             # Script de formatação personalizado
```

## 📋 Arquivos de Configuração por Categoria

### 🎨 Formatação e Linting
- **`rustfmt.toml`** - ⚠️ **DEVE ficar na raiz**
  - O Rust procura automaticamente na raiz do projeto
  - Alternativa: usar `--config-path` em scripts customizados
- **`.config/clippy.toml`** - Pode ser movido
  - Configuração do linter Clippy
  - Requer cópia para raiz no CI/CD
- **`.editorconfig`** - Recomendado na raiz
  - Configuração universal para editores
  - Funciona melhor na raiz do projeto

### 🏠 Editor e IDE
- **`.vscode/`** - Pasta padrão, manter localização
  - `settings.json` - Configurações do workspace
  - `extensions.json` - Extensões recomendadas
  - `launch.json` - Configurações de debug (se necessário)

### 🤖 CI/CD e Automação
- **`.github/`** - Pasta padrão do GitHub, manter localização
  - `workflows/ci.yml` - Pipeline de CI/CD
  - `dependabot.yml` - Atualizações automáticas
  - `CODEOWNERS` - Revisores automáticos (se necessário)

### 📦 Cargo e Rust
- **`Cargo.toml`** - ⚠️ **DEVE ficar na raiz**
  - Arquivo principal do workspace Rust
  - **`Cargo.lock`** - Gerado automaticamente, manter na raiz

## 🔧 Scripts e Utilitários

### `scripts/format.sh`
Script personalizado para formatação com configuração customizada:

```bash
#!/bin/bash
# Usa configuração do .config/rustfmt.toml
./scripts/format.sh          # Formatar código
./scripts/format.sh --check  # Verificar formatação
```

### Comandos Padrão
```bash
# Formatação (usa rustfmt.toml da raiz)
cargo fmt

# Linting (usa .config/clippy.toml se copiado)
cargo clippy

# Build
cargo build

# Testes
cargo test
```

## ✅ Recomendações

### ✅ Arquivos que PODEM ser movidos para `.config/`:
- `clippy.toml` - Configuração do Clippy
- `workspace-lints.toml` - Lints compartilhados
- `cargo-deny.toml` - Verificação de dependências
- `tarpaulin.toml` - Configuração de cobertura de código
- Custom scripts e configurações específicas do projeto

### ⚠️ Arquivos que DEVEM ficar na raiz:
- `rustfmt.toml` - Descoberta automática pelo Rust
- `Cargo.toml` - Padrão obrigatório do Cargo
- `Cargo.lock` - Gerado pelo Cargo
- `.gitignore` - Padrão do Git
- `.editorconfig` - Melhor descoberta na raiz

### 🏠 Pastas com localização fixa:
- `.vscode/` - Padrão do VS Code
- `.github/` - Padrão do GitHub
- `target/` - Gerada pelo Cargo

## 🚀 Benefícios da Organização

1. **📁 Centralização**: Configurações relacionadas em um local
2. **🧹 Limpeza**: Raiz do projeto menos poluída
3. **🔍 Descoberta**: Fácil localização de configurações
4. **🔧 Manutenção**: Facilitada gestão de configurações
5. **📚 Documentação**: Clara separação de responsabilidades

## 🎯 Implementação Atual

O projeto Toucan implementa esta organização com:
- ✅ Configurações centralizadas em `.config/`
- ✅ Scripts utilitários em `scripts/`
- ✅ Arquivos obrigatórios na raiz
- ✅ Integração com CI/CD
- ✅ Documentação clara
