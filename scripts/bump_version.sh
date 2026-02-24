#!/bin/bash
# Automated version bump script for FlashKraft workspace
#
# Called by `just bump <version>` which already runs the full quality-gate
# (fmt-check → clippy → tests) before invoking this script.
# Therefore this script does NOT re-run those checks — it only:
#   1. Updates [workspace.package] version in Cargo.toml
#   2. Updates the flashkraft-core workspace dependency version
#   3. Regenerates Cargo.lock
#   4. Generates/updates CHANGELOG.md
#   5. Commits everything
#   6. Creates an annotated git tag
#
# Usage:   ./scripts/bump_version.sh <new_version>
# Example: ./scripts/bump_version.sh 0.5.0

set -euo pipefail

# ── Colours ───────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# ── Argument validation ───────────────────────────────────────────────────────
if [ -z "${1:-}" ]; then
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
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${ROOT_DIR}"

# ── Read current version ──────────────────────────────────────────────────────
CURRENT_VERSION=$(grep -A5 '^\[workspace\.package\]' Cargo.toml \
    | grep '^version' \
    | head -1 \
    | sed 's/version *= *"\(.*\)"/\1/')

if [ -z "$CURRENT_VERSION" ]; then
    echo -e "${RED}Error: could not read current version from Cargo.toml${NC}"
    echo "Make sure the root Cargo.toml contains a [workspace.package] section with version = \"...\""
    exit 1
fi

if [ "$CURRENT_VERSION" = "$NEW_VERSION" ]; then
    echo -e "${YELLOW}Warning: new version (${NEW_VERSION}) is the same as the current version — nothing to do.${NC}"
    exit 0
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

# ── Check for an existing tag ─────────────────────────────────────────────────
if git rev-parse "v${NEW_VERSION}" >/dev/null 2>&1; then
    echo -e "${RED}Error: tag v${NEW_VERSION} already exists.${NC}"
    echo "Delete it first with:  git tag -d v${NEW_VERSION}"
    exit 1
fi

# ── 1. Update [workspace.package] version ────────────────────────────────────
echo -e "${GREEN}[1/4] Updating Cargo.toml [workspace.package] version…${NC}"
# awk is used instead of sed -i to work identically on macOS (BSD sed) and
# Linux (GNU sed) without needing different flags.
awk -v new_ver="${NEW_VERSION}" '
    /^\[workspace\.package\]/ { in_section=1 }
    in_section && /^\[/ && !/^\[workspace\.package\]/ { in_section=0 }
    in_section && /^version *= *"/ {
        sub(/"[^"]*"/, "\"" new_ver "\"")
        in_section=0
    }
    { print }
' Cargo.toml > Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml

# ── 2. Update the flashkraft-core workspace dependency version ────────────────
# The line looks like:
#   flashkraft-core = { path = "crates/flashkraft-core", version = "X.Y.Z" }
echo -e "${GREEN}[2/4] Updating flashkraft-core workspace dependency version…${NC}"
awk -v new_ver="${NEW_VERSION}" '
    /^flashkraft-core[[:space:]]*=/ {
        sub(/version[[:space:]]*=[[:space:]]*"[^"]*"/, "version = \"" new_ver "\"")
    }
    { print }
' Cargo.toml > Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml

# ── Verify all three crates inherit the workspace version ─────────────────────
# Each crate Cargo.toml must contain `version.workspace = true`.
MISSING_WORKSPACE_VERSION=0
for crate_toml in crates/*/Cargo.toml; do
    if ! grep -q 'version\.workspace *= *true' "${crate_toml}"; then
        echo -e "${YELLOW}Warning: ${crate_toml} does not use version.workspace = true${NC}"
        MISSING_WORKSPACE_VERSION=1
    fi
done
if [ "${MISSING_WORKSPACE_VERSION}" -eq 1 ]; then
    echo -e "${YELLOW}Some crates may not pick up the new version automatically.${NC}"
fi

# ── 3. Regenerate Cargo.lock ──────────────────────────────────────────────────
echo -e "${GREEN}[3/4] Regenerating Cargo.lock…${NC}"
cargo update --workspace 2>/dev/null || cargo generate-lockfile

# ── 4. Changelog ─────────────────────────────────────────────────────────────
echo -e "${GREEN}[4/4] Generating CHANGELOG.md…${NC}"
if command -v git-cliff &>/dev/null; then
    git-cliff --tag "v${NEW_VERSION}" -o CHANGELOG.md
    echo -e "${GREEN}CHANGELOG.md updated.${NC}"
else
    echo -e "${YELLOW}Warning: git-cliff not found — skipping changelog.${NC}"
    echo -e "${YELLOW}Install with: cargo install git-cliff${NC}"
fi

# ── Git: stage, commit, tag ───────────────────────────────────────────────────
echo ""
echo -e "${GREEN}Staging changes…${NC}"
git add Cargo.toml Cargo.lock CHANGELOG.md

echo -e "${GREEN}Creating commit…${NC}"
git commit -m "chore: bump version to ${NEW_VERSION}

- Update [workspace.package] version in Cargo.toml
- All crates inherit version via version.workspace = true
- Regenerate Cargo.lock
- Update CHANGELOG.md"

echo -e "${GREEN}Creating annotated tag v${NEW_VERSION}…${NC}"
git tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION}"

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}✅ Version bumped to ${NEW_VERSION} and tagged locally.${NC}"
echo ""
echo -e "${YELLOW}Next steps — push to trigger the CI release pipeline:${NC}"
echo ""
echo -e "  GitHub only:   ${CYAN}git push origin main && git push origin v${NEW_VERSION}${NC}"
echo -e "  Gitea only:    ${CYAN}git push gitea  main && git push gitea  v${NEW_VERSION}${NC}"
echo -e "  Both remotes:  ${CYAN}just push-release-all${NC}"
echo ""
echo -e "${YELLOW}Or use the just release shortcuts (these only push — bump already ran):${NC}"
echo -e "  ${CYAN}just push-release-all${NC}"
