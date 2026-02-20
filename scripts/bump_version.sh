#!/bin/bash
# Automated version bump script for FlashKraft workspace
# Usage: ./scripts/bump_version.sh <new_version>
# Example: ./scripts/bump_version.sh 0.5.0

set -e

# ── Colours ───────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# ── Argument validation ───────────────────────────────────────────────────────
if [ -z "$1" ]; then
    echo -e "${RED}Error: version number required${NC}"
    echo "Usage: $0 <version>"
    echo "Example: $0 0.5.0"
    exit 1
fi

NEW_VERSION="$1"

if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}Error: invalid version format '${NEW_VERSION}'${NC}"
    echo "Version must follow semantic versioning: X.Y.Z  (e.g. 0.5.0)"
    exit 1
fi

# ── Resolve workspace root ────────────────────────────────────────────────────
# The script may be called from any directory; always operate relative to the
# repository root (the directory that contains Cargo.toml with [workspace]).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${ROOT_DIR}"

# ── Read current version ──────────────────────────────────────────────────────
# The authoritative version lives in [workspace.package] in the root Cargo.toml.
CURRENT_VERSION=$(grep -A5 '^\[workspace\.package\]' Cargo.toml \
    | grep '^version' \
    | head -1 \
    | sed 's/version *= *"\(.*\)"/\1/')

if [ -z "$CURRENT_VERSION" ]; then
    echo -e "${RED}Error: could not read current version from Cargo.toml${NC}"
    echo "Make sure the root Cargo.toml contains a [workspace.package] section with version = \"...\""
    exit 1
fi

echo -e "${CYAN}Current version: ${CURRENT_VERSION}${NC}"
echo -e "${CYAN}New version:     ${NEW_VERSION}${NC}"
echo ""

# ── Confirmation ──────────────────────────────────────────────────────────────
read -p "Continue? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}Aborted.${NC}"
    exit 0
fi

# ── 1. Update [workspace.package] version ────────────────────────────────────
echo -e "${GREEN}[1/7] Updating Cargo.toml [workspace.package] version…${NC}"
# Use awk to only replace the version line that appears after [workspace.package]
# and before the next section header — avoids clobbering unrelated version fields.
awk -v new_ver="${NEW_VERSION}" '
    /^\[workspace\.package\]/ { in_section=1 }
    in_section && /^\[/ && !/^\[workspace\.package\]/ { in_section=0 }
    in_section && /^version *= *"/ {
        sub(/"[^"]*"/, "\"" new_ver "\"")
        in_section=0   # only replace the first occurrence
    }
    { print }
' Cargo.toml > Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml

# ── 2. Update the flashkraft-core workspace dependency version ────────────────
echo -e "${GREEN}[2/7] Updating flashkraft-core workspace dependency version…${NC}"
# The line looks like:  flashkraft-core = { path = "crates/flashkraft-core", version = "X.Y.Z" }
sed -i "s/\(flashkraft-core = {[^}]*version = \)\"[^\"]*\"/\1\"${NEW_VERSION}\"/" Cargo.toml

# ── 3. Update Cargo.lock ──────────────────────────────────────────────────────
echo -e "${GREEN}[3/7] Updating Cargo.lock…${NC}"
cargo update --workspace 2>/dev/null || cargo generate-lockfile

# ── 4. Formatting check ───────────────────────────────────────────────────────
echo -e "${GREEN}[4/7] Running cargo fmt…${NC}"
cargo fmt --all

# ── 5. Clippy ─────────────────────────────────────────────────────────────────
echo -e "${GREEN}[5/7] Running cargo clippy…${NC}"
if ! cargo clippy --workspace --all-targets --all-features -- -D warnings -A deprecated; then
    echo -e "${RED}Clippy found issues. Please fix them and re-run.${NC}"
    exit 1
fi

# ── 6. Tests ──────────────────────────────────────────────────────────────────
echo -e "${GREEN}[6/7] Running cargo test…${NC}"
if ! cargo test --workspace --locked --all-features --all-targets; then
    echo -e "${RED}Tests failed. Please fix them and re-run.${NC}"
    exit 1
fi

# ── 7. Changelog ─────────────────────────────────────────────────────────────
echo -e "${GREEN}[7/7] Generating CHANGELOG.md…${NC}"
if command -v git-cliff &>/dev/null; then
    git-cliff --tag "v${NEW_VERSION}" -o CHANGELOG.md
    echo -e "${GREEN}CHANGELOG.md updated.${NC}"
else
    echo -e "${YELLOW}Warning: git-cliff not found — skipping changelog.${NC}"
    echo -e "${YELLOW}Install with: cargo install git-cliff${NC}"
fi

# ── Git operations ────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}Staging changes…${NC}"
git add Cargo.toml Cargo.lock CHANGELOG.md

echo -e "${GREEN}Creating commit…${NC}"
git commit -m "chore: bump version to ${NEW_VERSION}

- Update [workspace.package] version in Cargo.toml
- Regenerate Cargo.lock
- Update CHANGELOG.md"

echo -e "${GREEN}Creating annotated tag v${NEW_VERSION}…${NC}"
git tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION}"

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}✅ Version bumped to ${NEW_VERSION} and tagged locally.${NC}"
echo ""
echo -e "${YELLOW}Push to remotes when ready:${NC}"
echo -e "  GitHub only:        ${CYAN}just release ${NEW_VERSION}${NC}"
echo -e "  Gitea only:         ${CYAN}just release-gitea ${NEW_VERSION}${NC}"
echo -e "  Both remotes:       ${CYAN}just release-all ${NEW_VERSION}${NC}"
echo ""
echo -e "${YELLOW}Or push manually:${NC}"
echo -e "  git push origin main && git push origin v${NEW_VERSION}"
