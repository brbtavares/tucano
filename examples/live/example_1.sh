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
# Carrega .env cedo para pegar DEBUG e credenciais (não falha se ausente)
ENV_FILE="$ROOT_DIR/.env"
if [[ -f "$ENV_FILE" ]]; then
  set -a
  # shellcheck disable=SC1090
  . "$ENV_FILE"
  set +a
fi
if [[ "${DEBUG:-}" == "1" ]]; then
  echo "[example_1.sh] DEBUG=1 ativado" >&2
  set -x
  export RUST_BACKTRACE=1
fi

# Ajuda específica se wine não estiver instalado (caso Arch Linux)
if ! command -v wine >/dev/null 2>&1; then
  if [[ -f /etc/arch-release || -f /etc/artix-release ]]; then
    echo "[example_1.sh][INFO] wine não encontrado. Instruções (Arch/Artix):" >&2
    echo "  1) Edite /etc/pacman.conf e habilite a seção [multilib] (descomente [multilib] e Include)." >&2
    echo "  2) sudo pacman -Syu" >&2
    echo "  3) sudo pacman -S wine winetricks mingw-w64-gcc" >&2
    echo "  (Opcional: sudo pacman -S wine-mono wine-gecko)" >&2
    echo "  Se 'winetricks' não estiver no repo principal, habilite 'extra' / use AUR (yay -S winetricks)." >&2
    echo "Abortando porque wine é necessário para backend real em Linux." >&2
    exit 127
  fi
fi
# Caminho padrão dentro do repo (se o usuário colocar manualmente ali)
DLL_DEFAULT="$ROOT_DIR/profitdll/ProfitDLL.dll"
# Se usuário tem DLL em $HOME/utils/ProfitDLL.dll e não definiu variável, usar como fallback secundário
USER_UTILS_DLL="$HOME/utils/ProfitDLL.dll"
if [[ -z "${PROFITDLL_PATH:-}" ]]; then
  if [[ -f "$DLL_DEFAULT" ]]; then
    export PROFITDLL_PATH="$DLL_DEFAULT"
  elif [[ -f "$USER_UTILS_DLL" ]]; then
    export PROFITDLL_PATH="$USER_UTILS_DLL"
  fi
fi
echo "[example_1.sh] PROFITDLL_PATH='${PROFITDLL_PATH:-<vazio>}'"
if [[ -z "${PROFITDLL_PATH:-}" || ! -f "${PROFITDLL_PATH}" ]]; then
  echo "ERRO: ProfitDLL.dll não encontrada. Opções:"
  echo "  1) export PROFITDLL_PATH=/caminho/para/ProfitDLL.dll" >&2
  echo "  2) colocar arquivo em $DLL_DEFAULT" >&2
  echo "  3) colocar arquivo em $USER_UTILS_DLL" >&2
  exit 1
fi
if [[ -z "${PROFIT_USER:-}" || -z "${PROFIT_PASSWORD:-}" ]]; then
  # Tenta auto-carregar .env se existir e ainda não carregado
  ENV_FILE="$ROOT_DIR/.env"
  if [[ -f "$ENV_FILE" ]]; then
    echo "[example_1.sh] Carregando credenciais de $ENV_FILE" >&2
    # Exporta variáveis definidas no .env
    set -a
    # shellcheck disable=SC1090
    . "$ENV_FILE"
    set +a
  fi
fi

# Ajusta caminho da DLL para execução via wine (se for caminho POSIX absoluto)
ORIG_DLL_POSIX="${PROFITDLL_PATH}"
if command -v wine >/dev/null 2>&1; then
  if [[ -n "${PROFITDLL_PATH:-}" && "${PROFITDLL_PATH}" == /* ]]; then
    WIN_PATH="Z:${PROFITDLL_PATH//\//\\}"
    echo "[example_1.sh] Convertendo caminho POSIX -> Windows: $PROFITDLL_PATH -> $WIN_PATH" >&2
    export PROFITDLL_PATH="$WIN_PATH"
  fi
fi

# Revalida após possível carregamento
if [[ -z "${PROFIT_USER:-}" || -z "${PROFIT_PASSWORD:-}" ]]; then
  echo "ERRO: Defina PROFIT_USER e PROFIT_PASSWORD. Opções:" >&2
  echo "  1) echo 'PROFIT_USER=seu_usuario' >> .env" >&2
  echo "     echo 'PROFIT_PASSWORD=seu_password' >> .env" >&2
  echo "  2) export PROFIT_USER=seu_usuario; export PROFIT_PASSWORD=seu_password" >&2
  echo "  3) Verifique se .env está no diretório raiz do repositório." >&2
  exit 1
fi
FEATURE_ARG="--features real_dll"
# Detecta arquitetura da DLL (se possível) para selecionar target 32/64-bit
WIN_TARGET="x86_64-pc-windows-gnu" # default 64-bit
if command -v file >/dev/null 2>&1 && [[ -n "${ORIG_DLL_POSIX:-}" && -f "${ORIG_DLL_POSIX}" ]]; then
  DLL_DESC=$(file -b "${ORIG_DLL_POSIX}" || true)
  echo "[example_1.sh] file => ${DLL_DESC}" >&2
  if grep -qi 'PE32 executable' <<<"$DLL_DESC" && ! grep -qi 'PE32+ executable' <<<"$DLL_DESC"; then
    WIN_TARGET="i686-pc-windows-gnu"
    echo "[example_1.sh] Detectado DLL 32-bit -> usando target ${WIN_TARGET}" >&2
  elif grep -qi 'PE32+ executable' <<<"$DLL_DESC"; then
    echo "[example_1.sh] Detectado DLL 64-bit" >&2
  else
    echo "[example_1.sh][AVISO] Arquitetura da DLL não determinada, mantendo ${WIN_TARGET}" >&2
  fi
fi
# Se estivermos em Linux e wine disponível, compila para target Windows e roda com wine.
if command -v wine >/dev/null 2>&1; then
  if ! rustup target list --installed | grep -q "^$WIN_TARGET$"; then
    echo "[example_1.sh] Instalando target $WIN_TARGET" >&2
    rustup target add $WIN_TARGET
  fi
  if [[ "${1:-}" != "--no-build" ]]; then
    cargo build -p tucano-examples $FEATURE_ARG --bin example_1_live_login --target $WIN_TARGET
  fi
  BIN_PATH="target/$WIN_TARGET/debug/example_1_live_login.exe"
  if [[ ! -f "$BIN_PATH" ]]; then
    echo "ERRO: binário não encontrado em $BIN_PATH" >&2
    exit 1
  fi
  echo "[example_1.sh] Executando via wine: $BIN_PATH" >&2
  if [[ "${DEBUG:-}" == "1" ]]; then
    export PROFITDLL_DIAG=1
    : "${WINEDEBUG:=err+loaddll}"
  else
    : "${WINEDEBUG:=-all}"
  fi
  echo "[example_1.sh] WINEDEBUG='$WINEDEBUG' PROFITDLL_PATH='$PROFITDLL_PATH'" >&2
  TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-30}"
  if [[ "${NO_TIMEOUT:-}" == "1" ]]; then
    echo "[example_1.sh] NO_TIMEOUT=1 -> executando sem timeout (cuidado com travas)." >&2
    wine "$BIN_PATH"
  else
    if command -v timeout >/dev/null 2>&1; then
      set +e
      timeout -k 5 "${TIMEOUT_SECONDS}" wine "$BIN_PATH"
      CODE=$?
      set -e
      if [[ $CODE -eq 124 ]]; then
        echo "[example_1.sh][AVISO] Execução excedeu ${TIMEOUT_SECONDS}s e foi interrompida." >&2
        echo "Possíveis causas: (1) DLL aguardando .NET (wine-mono) (2) Deadlock interno Initialize/Login (3) Falta de VC++ runtimes (4) Arquitetura mismatch 32/64-bit." >&2
        echo "Ações sugeridas:" >&2
        echo "  - Conferir arquitetura: file ${ORIG_DLL_POSIX}" >&2
        echo "  - Se DLL 32-bit: instalar wine 32-bit + target i686-pc-windows-gnu e dependências (multilib)." >&2
        echo "  - Instalar runtimes: winetricks -q wine-mono vcrun2019" >&2
        echo "  - Rodar sem timeout com mais logs: WINEDEBUG=+loaddll,+seh,+tid DEBUG=1 NO_TIMEOUT=1 ./examples/live/example_1.sh --no-build" >&2
        exit 124
      fi
    else
      wine "$BIN_PATH"
    fi
  fi
else
  # Fallback: execução nativa (somente terá backend real em Windows)
  if [[ "${1:-}" != "--no-build" ]]; then
    cargo build -p tucano-examples $FEATURE_ARG --bin example_1_live_login
  fi
  cargo run -p tucano-examples $FEATURE_ARG --bin example_1_live_login
fi
