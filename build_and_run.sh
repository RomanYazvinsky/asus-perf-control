#!/bin/bash
# Build and Run script for ASUS Linux Control
# Automatically builds Rust backend before running Flutter app

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== ASUS Linux Control Build Script ===${NC}"
echo ""

# Build Rust backend
echo -e "${YELLOW}[1/2] Building Rust backend...${NC}"
cd rust_core

if cargo build --release; then
    echo -e "${GREEN}✓ Rust backend built successfully${NC}"
else
    echo -e "${RED}✗ Failed to build Rust backend${NC}"
    exit 1
fi

cd ..

# Check that .so file exists
SO_FILE="rust_core/target/release/libasus_linux_control_core.so"
if [ ! -f "$SO_FILE" ]; then
    echo -e "${RED}✗ Error: $SO_FILE not found after build${NC}"
    exit 1
fi

echo -e "${GREEN}✓ FFI library ready: $SO_FILE${NC}"
echo ""

# Run Flutter app
echo -e "${YELLOW}[2/2] Running Flutter app...${NC}"
flutter run -d linux "$@"
