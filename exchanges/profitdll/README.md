<div align="center">


# toucan-profitdll

Isolated integration layer (types + abstractions + optional FFI) with **ProfitDLL** (Nelógica).

[![Crates.io](https://img.shields.io/crates/v/toucan-profitdll.svg)](https://crates.io/crates/toucan-profitdll)
[![Docs](https://img.shields.io/docsrs/toucan-profitdll)](https://docs.rs/toucan-profitdll)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../LICENSE)

</div>


> Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; Profit/ProfitDLL © Nelógica; see DISCLAIMER in the main repository.


## ✨ Overview

Provides:
- Events (`CallbackEvent`, `BookAction`, etc.)
- Identification types (`AssetIdentifier`, `AccountIdentifier`)
- Simple order sending (`SendOrder`)
- Cross-platform mock backend (Linux/macOS/Windows) for fast development
- (Optional via `real_dll` feature) dynamic loading of the real DLL (Windows)


## 🚀 Adding to `Cargo.toml`

```toml
[dependencies]
profitdll = { package = "toucan-profitdll", version = "0.1" }
```

To use the real DLL (Windows + installed/licensed DLL):

```toml
[dependencies]
profitdll = { package = "toucan-profitdll", version = "0.1", features = ["real_dll"] }
```


### Automatic Backend Selection
`new_backend()` tries:
1. Forces mock if `PROFITDLL_FORCE_MOCK=1`.
2. On Windows + `real_dll` feature, tries real DLL (uses `PROFITDLL_PATH` if set).
3. Fallback to mock.


## 🔐 Credentials / Environment Variables
```
PROFIT_USER=your_user
PROFIT_PASSWORD=your_password
PROFIT_ACTIVATION_KEY=optional
PROFITDLL_PATH=C:\path\ProfitDLL.dll   # optional
PROFITDLL_FORCE_MOCK=1                  # forces mock
```


## 🧪 Quick Example
```rust,no_run
use profitdll::{new_backend, SendOrder}; // alias for toucan-profitdll
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let backend = new_backend()?; // mock or real depending on environment
	let creds = profitdll::api::Credentials::from_env()?; // still accessible via alias
	let mut events = backend.initialize_login(&creds).await?;
	backend.subscribe_ticker("PETR4", "B")?;
	backend.send_order(&SendOrder { /* fields */ ..Default::default() })?;
	// consume events.try_recv() ...
	Ok(())
}
```


## 🧩 Feature Flags
- `real_dll`: enables FFI for the real DLL (Windows). Without it, only mock.


## 🛡️ Security
- FFI is inherently unsafe: validate inputs and handle all error codes.
- The real DLL is loaded dynamically; failures fall back to mock (with warning log).


## 🔁 API Stability
- Types marked with `#[non_exhaustive]` may receive future variants without breaking changes.
- Versions 0.x may introduce adjustments; pin if you need strict stability.


## 📄 Extended Documentation
See `MANUAL.md` (when present) and inline comments on main types.


## ⚖️ License & Trademarks
- Code under MIT (see `LICENSE`).
- Profit / ProfitDLL are trademarks and property of Nelógica; integration is purely technical.


---
Repeated Mini-Disclaimer: educational use; no recommendation; no affiliation; Profit/ProfitDLL © Nelógica.
