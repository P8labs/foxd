#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}════════════════════════════════════════════════${NC}"
echo -e "${GREEN}   Building foxd for all platforms${NC}"
echo -e "${BLUE}════════════════════════════════════════════════${NC}\n"

# Clean previous builds
rm -rf release
mkdir -p release

# Build Docker image with all targets
echo -e "${YELLOW}Building Docker image with all platforms...${NC}"
docker build -f Dockerfile.release -t foxd-builder:latest . || {
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
}

# Create a temporary container and copy binaries
echo -e "${YELLOW}Extracting binaries...${NC}"
CONTAINER_ID=$(docker create foxd-builder:latest)

# Copy all binaries from the container
docker cp ${CONTAINER_ID}:/ - | tar -xf - -C release/ 2>/dev/null || {
    # Alternative method: copy each file individually
    docker cp ${CONTAINER_ID}:/foxd-linux-amd64 release/
    docker cp ${CONTAINER_ID}:/foxd-linux-arm64 release/
    docker cp ${CONTAINER_ID}:/foxd-linux-armv7 release/
    docker cp ${CONTAINER_ID}:/foxd-windows-amd64.exe release/
}

# Clean up container
docker rm ${CONTAINER_ID} >/dev/null

# Make binaries executable
chmod +x release/foxd-* 2>/dev/null || true

# Show results
echo -e "\n${BLUE}════════════════════════════════════════════════${NC}"
echo -e "${GREEN}   Build complete!${NC}"
echo -e "${BLUE}════════════════════════════════════════════════${NC}\n"

echo -e "${YELLOW}Built binaries:${NC}"
ls -lh release/

echo -e "\n${YELLOW}Binary details:${NC}"
for binary in release/*; do
    if [ -f "$binary" ]; then
        echo -e "\n${GREEN}$(basename $binary):${NC}"
        file "$binary" 2>/dev/null || echo "  Binary: $(basename $binary)"
        if [ -x "$binary" ] && [[ "$binary" != *.exe ]]; then
            size=$(du -h "$binary" | cut -f1)
            echo "  Size: $size"
        fi
    fi
done

echo -e "\n${GREEN}✓ All builds successful!${NC}"
echo -e "${BLUE}Release binaries are in: ./release/${NC}\n"
