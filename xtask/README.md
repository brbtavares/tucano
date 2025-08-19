# xtask

```text
xtask/
├── Cargo.toml          ✅ Dependências completas
├── src/
│   ├── main.rs         ✅ Entry point e CLI
│   ├── app.rs          ✅ Estado da aplicação
│   ├── tui.rs          ✅ Interface TUI completa
│   ├── workspace.rs    ✅ Gerenciamento do workspace  
│   └── commands.rs     ✅ Implementação dos comandos
```

# Instale as dependências

```bash
cd xtask
cargo check  # Verifica se está tudo ok
```

## Execute a TUI

```bash
# From the workspace root:
cargo run --package xtask
# or specifically:
cargo run --package xtask -- tui
```

## Ou execute comandos diretamente

```bash
cargo run --package xtask -- fmt
cargo run --package xtask -- clippy
cargo run --package xtask -- size-check
cargo run --package xtask -- check-disclaimer
cargo run --package xtask -- add-disclaimer --fix
```
