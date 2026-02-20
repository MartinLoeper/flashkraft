# flashkraft workspace — task runner
# Install just: cargo install just
# Install git-cliff: cargo install git-cliff
# Usage: just <task>

# ── Default ───────────────────────────────────────────────────────────────────

default:
    @just --list

# ── Tool checks ───────────────────────────────────────────────────────────────

_check-git-cliff:
    @command -v git-cliff >/dev/null 2>&1 || { \
        echo "❌ git-cliff not found. Install with: cargo install git-cliff"; exit 1; \
    }

_check-cargo-edit:
    @command -v cargo-set-version >/dev/null 2>&1 || { \
        echo "❌ cargo-edit not found. Install with: cargo install cargo-edit"; exit 1; \
    }

# Install all recommended development tools
install-tools:
    @echo "Installing development tools…"
    @command -v git-cliff  >/dev/null 2>&1 || cargo install git-cliff
    @command -v cargo-edit >/dev/null 2>&1 || cargo install cargo-edit
    @echo "✅ All tools installed!"

# ── Build ─────────────────────────────────────────────────────────────────────

# Build the entire workspace (dev)
build:
    cargo build --workspace

# Build only the core library (dev)
build-core:
    cargo build -p flashkraft-core

# Build only the GUI crate (dev)
build-gui:
    cargo build -p flashkraft-gui

# Build only the TUI crate (dev)
build-tui:
    cargo build -p flashkraft-tui

# Build release binaries for GUI and TUI
build-release:
    cargo build --release -p flashkraft-gui
    cargo build --release -p flashkraft-tui

# Build a static (musl) TUI binary — great for portable distribution
build-tui-musl:
    @rustup target add x86_64-unknown-linux-musl 2>/dev/null || true
    cargo build --release -p flashkraft-tui --target x86_64-unknown-linux-musl
    @echo "✅ Static TUI binary: target/x86_64-unknown-linux-musl/release/flashkraft-tui"

# ── Run ───────────────────────────────────────────────────────────────────────

# Launch the Iced desktop GUI
run-gui:
    cargo run -p flashkraft-gui

# Launch the Ratatui terminal UI
run-tui:
    cargo run -p flashkraft-tui

# Alias: default run launches the TUI (headless-friendly)
run: run-tui

# ── Test ──────────────────────────────────────────────────────────────────────

# Run the full workspace test suite
test:
    cargo test --workspace --locked --all-features --all-targets

# Test only the core library
test-core:
    cargo test -p flashkraft-core --all-features

# Test only the GUI crate
test-gui:
    cargo test -p flashkraft-gui --all-features

# Test only the TUI crate
test-tui:
    cargo test -p flashkraft-tui --all-features

# ── Code quality ──────────────────────────────────────────────────────────────

# Check without building
check:
    cargo check --workspace

# Format all code
fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Run clippy across the workspace
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings -A deprecated

# Run all quality checks (fmt, clippy, test) — must pass before a release
check-all: fmt-check clippy test
    @echo "✅ All checks passed!"

# ── Documentation ─────────────────────────────────────────────────────────────

# Generate and open docs for the GUI crate
doc-gui:
    cargo doc --no-deps -p flashkraft-gui --open

# Generate and open docs for the TUI crate
doc-tui:
    cargo doc --no-deps -p flashkraft-tui --open

# Generate docs for the full workspace (no browser)
doc:
    cargo doc --no-deps --workspace

# ── Changelog ─────────────────────────────────────────────────────────────────

# Regenerate the full CHANGELOG.md from all tags
changelog: _check-git-cliff
    @echo "Generating full changelog…"
    git-cliff --output CHANGELOG.md
    @echo "✅ CHANGELOG.md updated."

# Prepend only unreleased commits to CHANGELOG.md
changelog-unreleased: _check-git-cliff
    git-cliff --unreleased --prepend CHANGELOG.md
    @echo "✅ Unreleased changes prepended."

# Preview changelog for the next release without writing the file
changelog-preview: _check-git-cliff
    @git-cliff --unreleased

# ── Version bump ─────────────────────────────────────────────────────────────
# Usage: just bump 0.5.0
#
# Runs fmt → clippy → test → changelog → commit → tag, then shows push hints.

bump version: check-all _check-git-cliff
    @echo "Bumping workspace version to {{version}}…"
    @./scripts/bump_version.sh {{version}}

# ── Publish (crates.io) ───────────────────────────────────────────────────────
# Publish order must be: core → gui → tui (dependency order).
# GUI and TUI are the only crates intended for public consumption; core is
# published as a prerequisite because cargo requires resolved version deps.

# Dry-run publish for all three crates (in dependency order)
publish-dry:
    @echo "Dry-run: flashkraft-core"
    cargo publish --dry-run -p flashkraft-core
    @echo "Dry-run: flashkraft-gui"
    cargo publish --dry-run -p flashkraft-gui
    @echo "Dry-run: flashkraft-tui"
    cargo publish --dry-run -p flashkraft-tui

# Publish flashkraft-core (prerequisite for the other two)
publish-core:
    @echo "📦 Publishing flashkraft-core…"
    cargo publish -p flashkraft-core
    @echo "⏳ Waiting 30 s for the index to propagate…"
    sleep 30

# Publish flashkraft-gui (requires core to already be on crates.io)
publish-gui:
    @echo "📦 Publishing flashkraft-gui…"
    cargo publish -p flashkraft-gui

# Publish flashkraft-tui (requires core to already be on crates.io)
publish-tui:
    @echo "📦 Publishing flashkraft-tui…"
    cargo publish -p flashkraft-tui

# Publish all three in the correct dependency order
publish: publish-core publish-gui publish-tui
    @echo "✅ All crates published to crates.io!"

# ── Housekeeping ──────────────────────────────────────────────────────────────

# Remove build artifacts
clean:
    cargo clean

# Update all dependencies
update:
    cargo update

# Show outdated dependencies (requires cargo-outdated)
outdated:
    cargo outdated

# Show the current workspace version
version:
    @grep -A5 '^\[workspace\.package\]' Cargo.toml \
        | grep '^version' \
        | head -1 \
        | sed 's/version *= *"\(.*\)"/\1/'

# Show project info
info:
    @echo "Project:   flashkraft"
    @echo "Version:   $(just version)"
    @echo "Author:    Sorin Irimies"
    @echo "License:   MIT"
    @echo ""
    @echo "Crates:"
    @echo "  flashkraft-core  — shared domain + flash engine (internal)"
    @echo "  flashkraft-gui   — Iced desktop GUI"
    @echo "  flashkraft-tui   — Ratatui terminal UI"

# ── Git helpers ───────────────────────────────────────────────────────────────

# Show configured remotes
remotes:
    @git remote -v

# Stage all changes and commit
commit message:
    git add -A
    git commit -m "{{message}}"

# Push the current branch to GitHub (origin)
push:
    git push origin main

# Push the current branch to Gitea
push-gitea:
    git push gitea main

# Push the current branch to both remotes
push-all:
    git push origin main
    git push gitea main
    @echo "✅ Pushed to both GitHub and Gitea!"

# Push all tags to GitHub
push-tags:
    git push origin --tags

# Push all tags to both remotes
push-tags-all:
    git push origin --tags
    git push gitea --tags
    @echo "✅ Tags pushed to both remotes!"

# ── Release workflows ─────────────────────────────────────────────────────────
# These combine bump + push so the CI release workflow fires automatically.

# Release to GitHub only: bump, commit, tag, push branch + tag
release version: (bump version)
    @echo "Pushing release v{{version}} to GitHub…"
    git push origin main
    git push origin "v{{version}}"
    @echo "✅ Release v{{version}} live on GitHub — CI will build & publish."

# Release to Gitea only
release-gitea version: (bump version)
    @echo "Pushing release v{{version}} to Gitea…"
    git push gitea main
    git push gitea "v{{version}}"
    @echo "✅ Release v{{version}} live on Gitea."

# Release to both GitHub and Gitea
release-all version: (bump version)
    @echo "Pushing release v{{version}} to all remotes…"
    git push origin main
    git push gitea main
    git push origin "v{{version}}"
    git push gitea "v{{version}}"
    @echo "✅ Release v{{version}} pushed to GitHub and Gitea!"

# Push the latest commit and all tags to every remote (no bump)
push-release-all:
    git push origin main
    git push gitea main
    git push origin --tags
    git push gitea --tags
    @echo "✅ Latest commit + tags pushed to all remotes."

# Force-sync Gitea with GitHub
sync-gitea:
    git push gitea main --force
    git push gitea --tags --force
    @echo "✅ Gitea force-synced with GitHub."

# Add a Gitea remote (provide full URL)
setup-gitea url:
    git remote add gitea {{url}}
    @echo "✅ Gitea remote added: {{url}}"
    @echo "Verify with: just remotes"
