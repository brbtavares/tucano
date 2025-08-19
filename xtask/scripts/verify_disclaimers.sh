#!/usr/bin/env bash
set -euo pipefail


# verify_disclaimers.sh - checks (or fixes with --fix) that all .rs files have the standard mini-disclaimer as the first line, and removes any old/duplicate disclaimers in the first 25 lines.
# Usage:
#   ./scripts/verify_disclaimers.sh          # only checks
#   ./scripts/verify_disclaimers.sh --fix    # fixes all .rs files
# Rules:
#   - The standard mini-disclaimer must be the first line of every .rs file.
#   - Any other disclaimer (old, duplicate, or in docstring) in the first 25 lines will be removed in --fix mode.
#   - If a suspicious pattern is found (e.g. disclaimer in the middle, or in a different format), a warning is printed for manual review.

DISCLAIMER_LINE='// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.'
DISCLAIMER_REGEX='^// Mini-Disclaimer:|^//! ?DISCLAIMER|^//! ?Mini-Disclaimer:|^//! ?DISCLAIMER \(summary\)|^// DISCLAIMER|^// Mini-Disclaimer:|^//! DISCLAIMER|^//! Mini-Disclaimer:|^//! ?DISCLAIMER \(resumo\)|^//! ?DISCLAIMER \(summary\)|^//! ?DISCLAIMER:|^//! ?DISCLAIMER'
SUSPICIOUS_KEYWORDS='(uso educacional|educational use|afiliac|investment advice|Profit/ProfitDLL|Nelógica|README|see README|resumo|summary|DISCLAIMER)'
MODE="verify"
[[ ${1-} == "--fix" ]] && MODE="fix"


missing=()
modified=0
manual_review=()

mapfile -t files < <(git ls-files '*.rs')


for f in "${files[@]}"; do
  [[ ! -s "$f" ]] && continue
  mapfile -t lines < "$f"
  # Check for suspicious patterns in the first 25 lines
  for ((i=0; i<${#lines[@]} && i<25; i++)); do
    if [[ "${lines[$i]}" =~ $SUSPICIOUS_KEYWORDS ]] && [[ ! "${lines[$i]}" =~ $DISCLAIMER_LINE ]]; then
      manual_review+=("$f:$((i+1)): ${lines[$i]}")
    fi
  done
  # Remove all disclaimers in the first 25 lines except the standard one (in fix mode)
  if [[ $MODE == fix ]]; then
    new_lines=()
    found_disclaimer=0
    # Se a primeira linha for um mini-disclaimer incorreto, remova
    if [[ "${lines[0]}" =~ $DISCLAIMER_REGEX && "${lines[0]}" != "$DISCLAIMER_LINE" ]]; then
      unset 'lines[0]'
      lines=("${lines[@]}")
    fi
    for ((i=0; i<${#lines[@]} && i<25; i++)); do
      if [[ "${lines[$i]}" == "$DISCLAIMER_LINE" && $found_disclaimer -eq 0 ]]; then
        new_lines+=("$DISCLAIMER_LINE")
        found_disclaimer=1
      elif [[ "${lines[$i]}" =~ $DISCLAIMER_REGEX ]]; then
        continue
      else
        new_lines+=("${lines[$i]}")
      fi
    done
    # Add the rest of the file
    for ((i=25; i<${#lines[@]}; i++)); do
      new_lines+=("${lines[$i]}")
    done
    # If disclaimer was missing, add it at the top
    if [[ $found_disclaimer -eq 0 ]]; then
      new_lines=("$DISCLAIMER_LINE" "" "${new_lines[@]}")
      modified=$((modified+1))
    fi
    printf '%s\n' "${new_lines[@]}" > "$f"
  else
    # Check mode: verify if the first line is the standard disclaimer
    if [[ "${lines[0]}" != "$DISCLAIMER_LINE" ]]; then
      missing+=("$f")
    fi
  fi
done


if [[ $MODE == fix ]]; then
  if (( modified > 0 )); then
    echo "[disclaimer] Inserted/fixed $modified mini-disclaimers."
  else
    echo "[disclaimer] No changes needed."
  fi
  if (( ${#manual_review[@]} > 0 )); then
    echo "[disclaimer] Manual review needed for suspicious lines:" >&2
    for m in "${manual_review[@]}"; do
      echo "  $m" >&2
    done
  fi
  exit 0
else
  if (( ${#missing[@]} > 0 )); then
    echo "[disclaimer] Missing mini-disclaimer in ${#missing[@]} file(s):" >&2
    for m in "${missing[@]}"; do
      echo "  - $m" >&2
    done
    echo "Use --fix to insert automatically." >&2
  fi
  if (( ${#manual_review[@]} > 0 )); then
    echo "[disclaimer] Manual review needed for suspicious lines:" >&2
    for m in "${manual_review[@]}"; do
      echo "  $m" >&2
    done
  fi
  if (( ${#missing[@]} == 0 && ${#manual_review[@]} == 0 )); then
    echo "[disclaimer] All files have the standard mini-disclaimer.";
  fi
  if (( ${#missing[@]} > 0 || ${#manual_review[@]} > 0 )); then
    exit 1
  fi
fi
