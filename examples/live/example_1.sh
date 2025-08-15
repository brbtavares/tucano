#!/usr/bin/env bash
# Runner para Example 1 (live login) exigindo DLL real.
# Uso:
#   examples/live/example_1.sh [--no-build]
# Pré-requisitos:
# - Feature real_dll
# - ProfitDLL.dll disponível (padrão: profitdll/ProfitDLL.dll ou PROFITDLL_PATH definido)
# - Variáveis: PROFIT_USER, PROFIT_PASSWORD (PROFIT_ACTIVATION_KEY opcional)
set -euo pipefail
ROOT_DIR="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
DLL_DEFAULT="$ROOT_DIR/profitdll/ProfitDLL.dll"
if [[ -z "${PROFITDLL_PATH:-}" && -f "$DLL_DEFAULT" ]]; then
  export PROFITDLL_PATH="$DLL_DEFAULT"
fi
if [[ ! -f "${PROFITDLL_PATH:-}" ]]; then
  echo "ERRO: ProfitDLL.dll não encontrada. Defina PROFITDLL_PATH ou coloque em $DLL_DEFAULT" >&2
  exit 1
fi
if [[ -z "${PROFIT_USER:-}" || -z "${PROFIT_PASSWORD:-}" ]]; then
  echo "ERRO: Defina PROFIT_USER e PROFIT_PASSWORD (export ou .env + dotenv)." >&2
  exit 1
fi
if [[ "${1:-}" != "--no-build" ]]; then
  cargo build -p tucano-examples --features real_dll --bin example_1_live_login
fi
cargo run -p tucano-examples --features real_dll --bin example_1_live_login
