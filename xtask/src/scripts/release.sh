#!/usr/bin/env bash
set -euo pipefail

# Simple release helper for toucan workspace.
# Usage: ./scripts/release.sh <crate> <new_version> [--publish]
# Example: ./scripts/release.sh toucan-core 0.12.3 --publish

CRATE=${1:-}
NEW_VERSION=${2:-}
PUBLISH_FLAG=${3:-}

if [[ -z "$CRATE" || -z "$NEW_VERSION" ]]; then
  echo "Usage: $0 <crate> <new_version> [--publish]" >&2
  exit 1
fi

if ! command -v cargo >/dev/null; then
  echo "cargo not found" >&2
  exit 1
fi

# Update version in the crate's Cargo.toml (package.version)
CRATE_DIR=$(grep -R "name = \"$CRATE\"" -n . | cut -d: -f1 | head -1 | xargs dirname)
if [[ ! -f "$CRATE_DIR/Cargo.toml" ]]; then
  echo "Could not locate crate directory for $CRATE" >&2
  exit 1
fi

# Use toml editing via sed (simple replace of version line)
sed -i "0,/^version = \"[0-9].*\"/s//version = \"$NEW_VERSION\"/" "$CRATE_DIR/Cargo.toml"

git add "$CRATE_DIR/Cargo.toml"
git commit -m "chore($CRATE): bump version to $NEW_VERSION"

tag_name="$CRATE-v$NEW_VERSION"
git tag -a "$tag_name" -m "$CRATE $NEW_VERSION"

echo "Created tag $tag_name"

if [[ "$PUBLISH_FLAG" == "--publish" ]]; then
  echo "Publishing $CRATE $NEW_VERSION..."
  cargo publish -p "$CRATE"
fi

echo "Done. Push with: git push origin main --tags"
