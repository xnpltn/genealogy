#!/bin/bash
set -euo pipefail

echo "BUILD: configuring...."
mkdir -p build
if [[ $? -ne 0 ]]; then
    echo "ERROR: Failed to create build directory."
    exit 1
fi

echo "BUILD: building..."
cargo build --release
if [[ $? -ne 0 ]]; then
    echo "ERROR: Cargo build failed."
    exit 1
fi

echo "BUILD: Copying files..."
cp -r ./target/release/genealogy ./build/
if [[ $? -ne 0 ]]; then
    echo "ERROR: Failed to copy the executable to the build directory."
    exit 1
fi

cp -r ./assets/ ./build/
if [[ $? -ne 0 ]]; then
    echo "ERROR: Failed to copy assets to the build directory."
    exit 1
fi

echo "BUILD: Successful!"
echo "INFO: Check in ./build directory."
