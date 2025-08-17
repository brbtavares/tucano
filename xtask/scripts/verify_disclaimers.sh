#!/usr/bin/env bash
set -euo pipefail
# verify_disclaimers.sh - verifica (ou corrige com --fix) presença de mini-disclaimer em arquivos .rs
# Uso:
#   ./scripts/verify_disclaimers.sh          # apenas verifica
#   ./scripts/verify_disclaimers.sh --fix    # injeta mini-disclaimer ausente
# Critério de presença: contém "Mini-Disclaimer:" OU "DISCLAIMER (resumo)" nas primeiras 25 linhas

DISCLAIMER_LINE='// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.'
MODE="verify"
[[ ${1-} == "--fix" ]] && MODE="fix"

missing=()
modified=0

mapfile -t files < <(git ls-files '*.rs')

for f in "${files[@]}"; do
  # pular arquivos vazios ou gerados (se algum dia houver padrão específico)
  [[ ! -s "$f" ]] && continue
  # detectar presença
  if head -n 25 "$f" | grep -qE 'Mini-Disclaimer:|DISCLAIMER \(resumo\)'; then
    continue
  fi
  # não tem
  if [[ $MODE == fix ]]; then
    tmp="$(mktemp)"
    # Inserir no topo preservando BOM (não esperado) e sem alterar permissões
    printf '%s\n' "$DISCLAIMER_LINE" > "$tmp"
    cat "$f" >> "$tmp"
    mv "$tmp" "$f"
    modified=$((modified+1))
  else
    missing+=("$f")
  fi
done

if [[ $MODE == fix ]]; then
  if (( modified > 0 )); then
    echo "[disclaimer] Inseridos $modified mini-disclaimers."
  else
    echo "[disclaimer] Nenhuma alteração necessária."
  fi
  exit 0
else
  if (( ${#missing[@]} > 0 )); then
    echo "[disclaimer] Faltando mini-disclaimer em ${#missing[@]} arquivo(s):" >&2
    for m in "${missing[@]}"; do
      echo "  - $m" >&2
    done
    echo "Use --fix para inserir automaticamente." >&2
    exit 1
  else
    echo "[disclaimer] Todos os arquivos possuem mini-disclaimer.";
  fi
fi
