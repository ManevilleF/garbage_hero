#!/bin/bash

set -e

CRATE_NAME="garbage_project"
OUT_DIR="web"
# Args parsing

# Default build profile, set with --profile=value
# The profile 'web-dev' is assumed to be a custom profile defined in Cargo.toml
BUILD_PROFILE="dev"

for arg in "$@"; do
	case $arg in
	--profile=*)
		BUILD_PROFILE="${arg#*=}"
		shift # Remove --profile=value from processing
		;;
	esac
done

echo "> Adding wasm target"
# Set the target for WebAssembly
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen-cli if it's not already installed
if ! command -v wasm-bindgen &>/dev/null; then
	echo "> Installing wasm-bindgen-cli"
	cargo install wasm-bindgen-cli --version 0.2.92
fi

echo "> Building $CRATE_NAME"
# Build the project with the specified or default profile
RUSTFLAGS='--cfg=web_sys_unstable_apis' \
cargo build \
	--lib \
	--profile "$BUILD_PROFILE" \
	--target wasm32-unknown-unknown

WASM_DIR="${OUT_DIR}/wasm"

echo "> Setting up build directories"
# Setup target directory
mkdir -p "$WASM_DIR"

# Move the assets
cp -r "assets" "$OUT_DIR"
cp "index.html" "$OUT_DIR"

echo "> Binding wasm build to $WASM_DIR"
# Bind the wasm build
wasm-bindgen --out-dir "$WASM_DIR" --target web "target/wasm32-unknown-unknown/${BUILD_PROFILE}/${CRATE_NAME}.wasm"

BINARY_PATH="${WASM_DIR}/${CRATE_NAME}_bg.wasm"

# Install wasm-opt if it's not already installed
if ! command -v wasm-opt &>/dev/null; then
	echo "Installing wasm-opt"
	cargo install wasm-opt
fi

echo "> Applying wasm optimizations"
# Apply wasm-specific optimizations to shrink the binary.
wasm-opt -O -ol 100 -s 100 -o "${WASM_DIR}/wasm-opt.wasm" "$BINARY_PATH" --enable-threads
rm "$BINARY_PATH"
mv "${WASM_DIR}/wasm-opt.wasm" "$BINARY_PATH"

# Print the size of the binary.
size="$(du -smh "$BINARY_PATH" | cut -f1)"
echo "Compiled binary is $size in size."
