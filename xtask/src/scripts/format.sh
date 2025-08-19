#!/bin/bash
# Tucano Project (formerly Toucan) - Format script
# Uses rustfmt with standard config in project root

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

if [ "$1" = "--check" ]; then
    echo "🔍 Checking code formatting..."
    cd "$PROJECT_ROOT" && cargo fmt --all -- --check
else
    echo "🎨 Formatting code..."
    cd "$PROJECT_ROOT" && cargo fmt --all
fi
