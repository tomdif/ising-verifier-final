#!/bin/bash
set -e

WASM_DIR="./wasm"
WASM_FILE="$WASM_DIR/vina.wasm"
WEBINA_URL="https://github.com/durrantlab/webina/releases/download/1.0.5/webina.zip"

echo "=== NEXUS Vina WASM Downloader ==="

mkdir -p "$WASM_DIR"

if [ -f "$WASM_FILE" ]; then
    echo "WASM binary already exists"
    sha256sum "$WASM_FILE"
    exit 0
fi

echo "Downloading Webina..."
TEMP_DIR=$(mktemp -d)
curl -L -o "$TEMP_DIR/webina.zip" "$WEBINA_URL"

echo "Extracting..."
unzip -q "$TEMP_DIR/webina.zip" -d "$TEMP_DIR"

VINA_WASM=$(find "$TEMP_DIR" -name "*.wasm" -type f | head -1)

if [ -z "$VINA_WASM" ]; then
    echo "Could not find vina.wasm"
    rm -rf "$TEMP_DIR"
    exit 1
fi

cp "$VINA_WASM" "$WASM_FILE"

echo "WASM binary ready:"
sha256sum "$WASM_FILE"

rm -rf "$TEMP_DIR"
