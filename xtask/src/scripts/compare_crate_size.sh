#!/usr/bin/env bash
# Compares the size (in MiB) of the local crate package with the version published on crates.io
# Uso: ./scripts/compare_crate_size.sh <crate> <versao>
# Exemplo: ./scripts/compare_crate_size.sh toucan-core 0.12.3
set -euo pipefail

CRATE=${1:-}
VERSION=${2:-}

if [[ -z "$CRATE" || -z "$VERSION" ]]; then
  echo "Uso: $0 <crate> <versao>" >&2
  exit 1
fi

# 1. Tamanho do pacote local
LOCAL_SIZE=$(cargo package --list -p "$CRATE" | awk '{print $2}' | xargs -I{} du -b {} 2>/dev/null | awk '{s+=$1} END {print s}')

# 2. Baixar e extrair o .crate publicado
TMPDIR=$(mktemp -d)
CRATE_FILE="$TMPDIR/$CRATE-$VERSION.crate"
curl -sSL -o "$CRATE_FILE" "https://crates.io/api/v1/crates/$CRATE/$VERSION/download"
tar -xf "$CRATE_FILE" -C "$TMPDIR"
PUBLISHED_SIZE=$(find "$TMPDIR" -type f ! -name "*.crate" -exec du -b {} + | awk '{s+=$1} END {print s}')

# 3. Converter para MiB
LOCAL_MIB=$(awk "BEGIN {printf \"%.2f\", $LOCAL_SIZE/1024/1024}")
PUBLISHED_MIB=$(awk "BEGIN {printf \"%.2f\", $PUBLISHED_SIZE/1024/1024}")
DIFF_MIB=$(awk "BEGIN {printf \"%.2f\", $LOCAL_MIB-$PUBLISHED_MIB}")

# 4. Exibir resultado
printf "Local:      %7.2f MiB\n" "$LOCAL_MIB"
printf "Publicado:  %7.2f MiB\n" "$PUBLISHED_MIB"
printf "Difference:  %7.2f MiB\n" "$DIFF_MIB"

rm -rf "$TMPDIR"
