#!/bin/bash

# Script to update the Homebrew formula with correct SHA256 hashes
# Usage: ./scripts/update-formula.sh v0.1.0

set -e

VERSION=${1:-"v0.1.0"}
FORMULA_FILE="Formula/gthr.rb"

echo "Updating formula for version $VERSION..."

# Download and calculate SHA256 for each platform
echo "Calculating SHA256 hashes..."

# macOS ARM64
ARM64_URL="https://github.com/Adarsh-Roy/gthr/releases/download/$VERSION/gthr-aarch64-apple-darwin.tar.gz"
ARM64_SHA=$(curl -sL "$ARM64_URL" | shasum -a 256 | cut -d' ' -f1)
echo "ARM64 SHA256: $ARM64_SHA"

# macOS x86_64
X86_64_URL="https://github.com/Adarsh-Roy/gthr/releases/download/$VERSION/gthr-x86_64-apple-darwin.tar.gz"
X86_64_SHA=$(curl -sL "$X86_64_URL" | shasum -a 256 | cut -d' ' -f1)
echo "x86_64 SHA256: $X86_64_SHA"

# Linux x86_64
LINUX_URL="https://github.com/Adarsh-Roy/gthr/releases/download/$VERSION/gthr-x86_64-unknown-linux-gnu.tar.gz"
LINUX_SHA=$(curl -sL "$LINUX_URL" | shasum -a 256 | cut -d' ' -f1)
echo "Linux SHA256: $LINUX_SHA"

# Update the formula file
sed -i.bak \
  -e "s/version \".*\"/version \"${VERSION#v}\"/" \
  -e "s|download/v[0-9.]*/|download/$VERSION/|g" \
  -e "s/REPLACE_WITH_ACTUAL_SHA256_FOR_ARM64/$ARM64_SHA/" \
  -e "s/REPLACE_WITH_ACTUAL_SHA256_FOR_X86_64/$X86_64_SHA/" \
  -e "s/REPLACE_WITH_ACTUAL_SHA256_FOR_LINUX/$LINUX_SHA/" \
  "$FORMULA_FILE"

echo "Formula updated successfully!"
echo "Please review the changes in $FORMULA_FILE"