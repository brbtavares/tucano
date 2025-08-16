#!/usr/bin/env bash
set -euo pipefail

# Local pre-commit quality gate for Tucano workspace.
# Run formatting, lint, deny, tests (fast subset) before allowing commit.
# Usage (install hook): ln -sf ../../scripts/pre_commit.sh .git/hooks/pre-commit
# Or run manually: ./scripts/pre_commit.sh

echo "[pre-commit] Starting checks..."

if ! command -v cargo >/dev/null; then
  echo "cargo not found" >&2
  exit 1
fi

echo "[pre-commit] Formatting (rustfmt)..."
cargo fmt --all -- --check

echo "[pre-commit] Clippy (all targets, all features)..."
cargo clippy --all-targets --all-features --quiet -- -D warnings

echo "[pre-commit] Cargo deny (advisories, bans, licenses, sources)..."
if command -v cargo-deny >/dev/null; then
  cargo deny check advisories licenses bans sources
else
  echo "[pre-commit] cargo-deny not installed (skip) -> install with: cargo install cargo-deny --locked" >&2
fi


echo "[pre-commit] Verificando disclaimers..."
./scripts/verify_disclaimers.sh

echo "[pre-commit] Tests (fast) ..."
# Strategy: run doc tests + library/unit tests w/out features that increase build time.
cargo test --all --lib --quiet

echo "[pre-commit] Completed successfully."
