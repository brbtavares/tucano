#!/usr/bin/env bash
# Sequential publish script for Tucano crates.
# Usage: ./scripts/publish_sequence.sh
# Requires: CARGO_REGISTRY_TOKEN (cargo login) already configured.
# Aborts on first failure.
set -euo pipefail

ORDER=(
  "tucano-markets"
  "tucano-integration"
  "tucano-data"
  "tucano-execution"
  "tucano-trader"
  "tucano-risk"
  "tucano-analytics"
  "tucano-core"
  "tucano-strategies"
  "tucano"
)

step() {
  local crate=$1
  echo "\n=== Packaging $crate ==="
  cargo package -p "$crate" || { echo "Package failed for $crate"; exit 1; }
  echo "\n=== Publishing $crate ==="
  cargo publish -p "$crate" || { echo "Publish failed for $crate"; exit 1; }
  echo "Waiting for index update (sleep 30s)..."
  sleep 30
}

echo "Tucano publish sequence start"

for crate in "${ORDER[@]}"; do
  step "$crate"
  echo "Done: $crate"
  echo "----------------------------------------"
  # Optional manual pause: uncomment below if you want to confirm each step
  # read -rp "Press enter to continue..." _

done

echo "All crates published."
