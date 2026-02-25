#!/bin/bash
# Pre-publish readiness check for the FlashKraft workspace.
# Run this before pushing a release tag to catch problems early.
#
# Usage:   ./scripts/check_publish.sh
# Or via just:
#   just check-publish

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${ROOT_DIR}"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

errors=0

pass() { echo -e "${GREEN}✓${NC}  $1"; }
fail() { echo -e "${RED}✗${NC}  $1"; errors=$((errors + 1)); }
warn() { echo -e "${YELLOW}⚠${NC}  $1"; }
section() { echo ""; echo -e "${CYAN}── $1 ──${NC}"; }

echo ""
echo -e "${CYAN}════════════════════════════════════════${NC}"
echo -e "${CYAN}  FlashKraft — Publish Readiness Check${NC}"
echo -e "${CYAN}════════════════════════════════════════${NC}"

# ── 1. Formatting ─────────────────────────────────────────────────────────────
section "Formatting"
echo -n "  cargo fmt --all -- --check ... "
if cargo fmt --all -- --check > /dev/null 2>&1; then
    pass "code is formatted"
else
    fail "formatting issues found  (run: cargo fmt --all)"
fi

# ── 2. Clippy ─────────────────────────────────────────────────────────────────
section "Clippy"
echo -n "  cargo clippy --workspace ... "
if cargo clippy --workspace --all-targets --all-features -- -D warnings -A deprecated > /dev/null 2>&1; then
    pass "no clippy warnings"
else
    fail "clippy found issues  (run: cargo clippy --workspace --all-targets --all-features -- -D warnings)"
fi

# ── 3. Tests ──────────────────────────────────────────────────────────────────
section "Tests"
echo -n "  cargo test --workspace ... "
if cargo test --workspace --all-features --all-targets > /dev/null 2>&1; then
    pass "all tests pass"
else
    fail "test failures found  (run: cargo test --workspace --all-features)"
fi

# ── 4. Documentation ──────────────────────────────────────────────────────────
section "Documentation"
for crate in flashkraft-core flashkraft-gui flashkraft-tui; do
    echo -n "  cargo doc -p ${crate} ... "
    if cargo doc --no-deps -p "${crate}" --all-features > /dev/null 2>&1; then
        pass "${crate}"
    else
        fail "${crate}  (run: cargo doc --no-deps -p ${crate})"
    fi
done

# ── 5. Required files ─────────────────────────────────────────────────────────
section "Required files"
for file in README.md LICENSE Cargo.toml CHANGELOG.md cliff.toml; do
    echo -n "  ${file} ... "
    if [ -f "${file}" ]; then
        pass "present"
    else
        fail "missing"
    fi
done

# ── 6. Workspace version consistency ─────────────────────────────────────────
section "Workspace version consistency"
WORKSPACE_VERSION=$(grep -A5 '^\[workspace\.package\]' Cargo.toml \
    | grep '^version' \
    | head -1 \
    | sed 's/version *= *"\(.*\)"/\1/')

if [ -z "${WORKSPACE_VERSION}" ]; then
    fail "could not read [workspace.package] version from Cargo.toml"
else
    echo -n "  workspace version: ${WORKSPACE_VERSION} ... "
    pass "found"
fi

all_inherit=true
for crate_toml in crates/*/Cargo.toml; do
    crate_name=$(basename "$(dirname "${crate_toml}")")
    echo -n "  ${crate_name} uses version.workspace ... "
    if grep -q 'version\.workspace *= *true' "${crate_toml}"; then
        pass "yes"
    else
        warn "${crate_name} does not set version.workspace = true"
        all_inherit=false
    fi
done

# ── 7. Cargo.lock is up to date ───────────────────────────────────────────────
section "Cargo.lock"
echo -n "  Cargo.lock present ... "
if [ -f "Cargo.lock" ]; then
    pass "present"
else
    fail "missing — run: cargo generate-lockfile"
fi

# ── 8. Publish readiness ──────────────────────────────────────────────────────
# flashkraft-core: full publish dry-run — it has no workspace path deps, so
#   cargo can resolve everything against the crates.io index.
#
# flashkraft-gui / flashkraft-tui: these depend on flashkraft-core via a path
#   dep.  Both `cargo publish --dry-run` and `cargo package` require all
#   transitive deps to be resolvable on crates.io — which is only true *after*
#   core is published.  We therefore verify them with `cargo check` instead;
#   the full packaging is validated by the CI release workflow which publishes
#   in the correct order (core → gui → tui).
section "Publish readiness"

echo -n "  cargo publish --dry-run --allow-dirty -p flashkraft-core ... "
if cargo publish --dry-run --allow-dirty -p flashkraft-core > /dev/null 2>&1; then
    pass "flashkraft-core"
else
    fail "flashkraft-core  (run: cargo publish --dry-run --allow-dirty -p flashkraft-core for details)"
fi

for crate in flashkraft-gui flashkraft-tui; do
    echo -n "  cargo check -p ${crate} (packaging verified by CI) ... "
    if cargo check -p "${crate}" > /dev/null 2>&1; then
        pass "${crate}"
    else
        fail "${crate}  (run: cargo check -p ${crate} for details)"
    fi
done

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}════════════════════════════════════════${NC}"
if [ "${errors}" -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed — ready to release! 🚀${NC}"
    echo ""
    echo -e "${CYAN}Next step:${NC}"
    echo -e "  just bump <version>   # e.g. just bump 0.5.0"
    exit 0
else
    echo -e "${RED}✗ ${errors} check(s) failed — please fix before releasing.${NC}"
    exit 1
fi
