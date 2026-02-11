#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

CARGO_TOML="daemon/Cargo.toml"
PACKAGE_JSON="console/package.json"

# Get current version from Cargo.toml
get_version() {
    grep '^version = ' "$CARGO_TOML" | head -n1 | sed 's/version = "\(.*\)"/\1/'
}

# Parse version into components
parse_version() {
    local version=$1
    MAJOR=$(echo "$version" | cut -d. -f1)
    MINOR=$(echo "$version" | cut -d. -f2)
    PATCH=$(echo "$version" | cut -d. -f3)
}

# Bump version based on type
bump_version() {
    local bump_type=$1
    parse_version "$CURRENT_VERSION"
    
    case $bump_type in
        major)
            MAJOR=$((MAJOR + 1))
            MINOR=0
            PATCH=0
            ;;
        minor)
            MINOR=$((MINOR + 1))
            PATCH=0
            ;;
        patch)
            PATCH=$((PATCH + 1))
            ;;
        *)
            echo -e "${RED}Invalid bump type. Use: major, minor, or patch${NC}"
            exit 1
            ;;
    esac
    
    echo "$MAJOR.$MINOR.$PATCH"
}

# Update version in Cargo.toml
update_cargo_version() {
    local new_version=$1
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
    else
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
    fi
}

# Update version in package.json
update_package_version() {
    local new_version=$1
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/\"version\": \".*\"/\"version\": \"$new_version\"/" "$PACKAGE_JSON"
    else
        sed -i "s/\"version\": \".*\"/\"version\": \"$new_version\"/" "$PACKAGE_JSON"
    fi
}

# Show binary information
show_info() {
    local version=$1
    echo -e "\n${BLUE}═══════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}   foxd - LAN Monitoring Daemon${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
    echo -e "${YELLOW}Version:${NC}     $version"
    echo -e "${YELLOW}Binary:${NC}      foxd"
    echo -e "${YELLOW}Platform:${NC}    Linux, Windows (x86_64, ARM64, ARMv7)"
    echo -e "${YELLOW}Language:${NC}    Rust"
    echo -e "${YELLOW}Console:${NC}     SvelteKit"
    echo -e "${BLUE}═══════════════════════════════════════════════════${NC}\n"
}

# Create git tag
create_tag() {
    local version=$1
    local tag="v$version"
    
    echo -e "${YELLOW}Creating git tag: $tag${NC}"
    git add "$CARGO_TOML" "$PACKAGE_JSON"
    git commit -m "chore: bump version to $version"
    git tag -a "$tag" -m "Release $version"
    
    echo -e "${GREEN}✓ Tag created: $tag${NC}\n"
    echo -e "${BLUE}To push the tag and trigger release:${NC}"
    echo -e "${YELLOW}  git push origin main && git push origin $tag${NC}\n"
}

# Main script
main() {
    if [ $# -eq 0 ]; then
        echo -e "${RED}Usage: $0 <major|minor|patch>${NC}"
        echo -e "Example: $0 patch"
        exit 1
    fi
    
    BUMP_TYPE=$1
    CURRENT_VERSION=$(get_version)
    
    echo -e "${BLUE}Current version:${NC} $CURRENT_VERSION"
    
    NEW_VERSION=$(bump_version "$BUMP_TYPE")
    
    echo -e "${GREEN}New version:${NC}     $NEW_VERSION"
    echo ""
    read -p "Continue with version bump? (y/N) " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${RED}Aborted.${NC}"
        exit 1
    fi
    
    echo -e "\n${YELLOW}Updating versions...${NC}"
    update_cargo_version "$NEW_VERSION"
    update_package_version "$NEW_VERSION"
    echo -e "${GREEN}✓ Updated Cargo.toml${NC}"
    echo -e "${GREEN}✓ Updated package.json${NC}"
    
    show_info "$NEW_VERSION"
    
    read -p "Create git tag and commit? (y/N) " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        create_tag "$NEW_VERSION"
    else
        echo -e "${YELLOW}Version files updated but no git tag created.${NC}"
        echo -e "${BLUE}Run the following when ready:${NC}"
        echo -e "${YELLOW}  git add $CARGO_TOML $PACKAGE_JSON${NC}"
        echo -e "${YELLOW}  git commit -m 'chore: bump version to $NEW_VERSION'${NC}"
        echo -e "${YELLOW}  git tag -a v$NEW_VERSION -m 'Release $NEW_VERSION'${NC}"
        echo -e "${YELLOW}  git push origin master && git push origin v$NEW_VERSION${NC}"
    fi
}

main "$@"
