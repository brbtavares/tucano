#!/bin/bash
# Toucan Project - Format script
# Uses rustfmt with custom config path

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
CONFIG_PATH="$SCRIPT_DIR/.config/rustfmt.toml"

if [ "$1" = "--check" ]; then
    echo "🔍 Checking code formatting..."
    cargo fmt --all -- --config-path="$CONFIG_PATH" --check
else
    echo "🎨 Formatting code..."
    cargo fmt --all -- --config-path="$CONFIG_PATH"
fi
