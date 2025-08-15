<div align="center">

# profitdll

Camada isolada de integração (tipos + abstrações + FFI opcional) com a **ProfitDLL** (Nelógica).

[![Crates.io](https://img.shields.io/crates/v/profitdll.svg)](https://crates.io/crates/profitdll)
[![Docs](https://img.shields.io/docsrs/profitdll)](https://docs.rs/profitdll)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../LICENSE)

</div>

> Mini-Disclaimer: uso educacional/experimental; sem recomendação de investimento ou afiliação; Profit/ProfitDLL © Nelógica; veja DISCLAIMER no repositório principal.

## ✨ Visão Geral

Fornece:
- Eventos (`CallbackEvent`, `BookAction`, etc.)
- Tipos de identificação (`AssetIdentifier`, `AccountIdentifier`)
- Envio simples de ordens (`SendOrder`)
- Backend mock multiplataforma (Linux/macOS/Windows) para desenvolvimento rápido
- (Opcional via feature `real_dll`) carregamento dinâmico da DLL real (Windows)

## 🚀 Adicionando ao `Cargo.toml`

```toml
[dependencies]
profitdll = "0.1"
```

Para usar a DLL real (Windows + DLL instalada/licenciada):

```toml
[dependencies]
profitdll = { version = "0.1", features = ["real_dll"] }
```

### Seleção Automática de Backend
`new_backend()` tenta:
1. Forçar mock se `PROFITDLL_FORCE_MOCK=1`.
2. Em Windows + feature `real_dll`, tenta DLL real (usa `PROFITDLL_PATH` se definido).
3. Fallback para mock.

## 🔐 Credenciais / Variáveis de Ambiente
```
PROFIT_USER=seu_usuario
PROFIT_PASSWORD=sua_senha
PROFIT_ACTIVATION_KEY=opcional
PROFITDLL_PATH=C:\\caminho\\ProfitDLL.dll   # opcional
PROFITDLL_FORCE_MOCK=1                        # força mock
```

## 🧪 Exemplo Rápido
```rust,no_run
use profitdll::{new_backend, SendOrder};
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let backend = new_backend()?; // mock ou real dependendo do ambiente
	let creds = profitdll::api::Credentials::from_env()?;
	let mut events = backend.initialize_login(&creds).await?;
	backend.subscribe_ticker("PETR4", "B")?;
	backend.send_order(&SendOrder { /* campos */ ..Default::default() })?;
	// consumir events.try_recv() ...
	Ok(())
}
```

## 🧩 Feature Flags
- `real_dll`: habilita FFI para a DLL real (Windows). Sem ela, apenas mock.

## 🛡️ Segurança
- FFI é inerentemente inseguro: valide entradas e trate todos os códigos de erro.
- A DLL real é carregada dinamicamente; falhas fazem fallback para o mock (log de aviso).

## 🔁 Estabilidade da API
- Tipos marcados com `#[non_exhaustive]` podem receber variantes futuras sem breaking change.
- Versões 0.x podem introduzir ajustes; pin se precisar de estabilidade rígida.

## 📄 Documentação Estendida
Consulte `MANUAL.md` (quando presente) e os comentários inline dos tipos principais.

## ⚖️ Licença & Marcas
- Código sob MIT (ver `LICENSE`).
- Profit / ProfitDLL são marcas e propriedade da Nelógica; integração meramente técnica.

---
Mini-Disclaimer repetido: uso educacional; sem recomendação; sem afiliação; Profit/ProfitDLL © Nelógica.
