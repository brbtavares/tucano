#!/usr/bin/env bash
# Run ProfitDLL (real_dll feature) tests on Linux by cross-compiling to Windows and executing under Wine.
# Usage:
#   scripts/test_live_wine.sh /caminho/para/ProfitDLL.dll [extra_cargo_args]
#   scripts/test_live_wine.sh --build-only              # só compila para Windows (validação de compile)
# Required env vars (export antes se quiser rodar testes que exigem login):
#   PROFIT_USER, PROFIT_PASSWORD (PROFIT_ACTIVATION_KEY opcional)
set -euo pipefail

BUILD_ONLY=0
if [[ ${1:-} == "--build-only" ]]; then
  BUILD_ONLY=1
  shift || true
fi

if [[ $BUILD_ONLY -eq 0 ]]; then
  if [[ ${1:-} == "" ]]; then
    echo "ERRO: informe caminho da ProfitDLL.dll como primeiro argumento ou use --build-only" >&2
    exit 1
  fi
  DLL_SRC=$1; shift || true
  if [[ ! -f "$DLL_SRC" ]]; then
    echo "ERRO: arquivo DLL não encontrado: $DLL_SRC" >&2
    exit 1
  fi
fi

if [[ $BUILD_ONLY -eq 0 ]]; then
  if ! command -v wine >/dev/null 2>&1; then
    echo "ERRO: wine não instalado. Instale (ex: sudo apt-get install wine64)." >&2
    exit 1
  fi
fi
if ! rustup target list | grep -q '^x86_64-pc-windows-gnu (installed)'; then
  echo "Instalando target x86_64-pc-windows-gnu..." >&2
  rustup target add x86_64-pc-windows-gnu
fi

TARGET_DIR=target/x86_64-pc-windows-gnu/debug
DLL_DEST=$TARGET_DIR/ProfitDLL.dll
mkdir -p "$TARGET_DIR"
if [[ $BUILD_ONLY -eq 0 ]]; then
  cp "$DLL_SRC" "$DLL_DEST"
  export PROFITDLL_PATH="ProfitDLL.dll"  # usado pelo wrapper
  export WINEDEBUG=-all
fi

echo "[1/3] Compilando testes (target windows)..."
cxx_tool_missing=0
for tool in x86_64-w64-mingw32-gcc x86_64-w64-mingw32-dlltool; do
  if ! command -v $tool >/dev/null 2>&1; then
    cxx_tool_missing=1
  fi
done
if [[ $cxx_tool_missing -eq 1 ]]; then
  echo "AVISO: toolchain MinGW incompleta (instale: sudo apt-get install mingw-w64)." >&2
fi
cargo test -p tucano-profitdll --features real_dll --target x86_64-pc-windows-gnu --no-run "$@"

if [[ $BUILD_ONLY -eq 1 ]]; then
  echo "[2/2] (build-only) Sucesso: testes compilados para Windows. Para execução: remova --build-only e forneça DLL."
  exit 0
fi

echo "[2/3] Localizando executáveis de teste..."
mapfile -t TEST_EXES < <(find "$TARGET_DIR/deps" -maxdepth 1 -type f -name "*.exe" -perm -111 | sort)
if [[ ${#TEST_EXES[@]} -eq 0 ]]; then
  echo "Nenhum executável de teste encontrado." >&2
  exit 1
fi

FAIL=0
TOTAL=${#TEST_EXES[@]}
IDX=0
echo "[3/3] Executando $TOTAL binários de teste via wine..."
for exe in "${TEST_EXES[@]}"; do
  ((IDX++)) || true
  BN=$(basename "$exe")
  echo "--> ($IDX/$TOTAL) $BN"
  if ! wine "$exe" --color always --nocapture; then
    echo "FALHA: $BN" >&2
    FAIL=1
  fi
done

if [[ $FAIL -ne 0 ]]; then
  echo "Alguns testes falharam." >&2
  exit 1
fi

echo "Todos os testes passaram sob wine."
