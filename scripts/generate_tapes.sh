#!/bin/bash
# Generate all FlashKraft VHS demos
# This script runs all VHS tapes and generates GIFs and screenshots

set -e

# Get the project root directory (parent of scripts/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Change to project root
cd "$PROJECT_ROOT"

echo "╔════════════════════════════════════════════╗"
echo "║   FlashKraft VHS Demo Generator           ║"
echo "╚════════════════════════════════════════════╝"
echo ""

# Check if VHS is installed
if ! command -v vhs &> /dev/null; then
    echo "❌ VHS is not installed!"
    echo ""
    echo "Install it with:"
    echo "  brew install vhs"
    echo "  or"
    echo "  go install github.com/charmbracelet/vhs@latest"
    echo ""
    exit 1
fi

echo "✓ VHS is installed"
echo ""

# Create screenshots directory if it doesn't exist
mkdir -p examples/vhs/screenshots

# List of tapes to process
TAPES=(
    "demo-quick"
    "demo-basic"
    "demo-themes"
    "demo-workflow"
    "demo-examples"
    "demo-build"
    "demo-architecture"
    "demo-features"
    "demo-storage"
)

# Process each tape
for tape in "${TAPES[@]}"; do
    TAPE_FILE="${PROJECT_ROOT}/examples/vhs/${tape}.tape"

    if [ -f "$TAPE_FILE" ]; then
        echo "─────────────────────────────────────────────"
        echo "Processing: ${tape}.tape"
        echo "─────────────────────────────────────────────"

        # Run VHS
        if vhs "$TAPE_FILE"; then
            echo "✓ Generated: ${PROJECT_ROOT}/examples/vhs/${tape}.gif"
        else
            echo "❌ Failed to process ${tape}.tape"
        fi
        echo ""
    else
        echo "⚠ Warning: ${TAPE_FILE} not found"
        echo ""
    fi
done

echo "╔════════════════════════════════════════════╗"
echo "║   Generation Complete!                     ║"
echo "╚════════════════════════════════════════════╝"
echo ""
echo "Generated files:"
echo "  📁 ${PROJECT_ROOT}/examples/vhs/"
ls -lh "${PROJECT_ROOT}/examples/vhs/"*.gif 2>/dev/null || echo "    (no GIF files generated)"
echo ""
echo "  📁 ${PROJECT_ROOT}/examples/vhs/screenshots/"
ls -1 "${PROJECT_ROOT}/examples/vhs/screenshots/"*.png 2>/dev/null | wc -l | xargs echo "    Screenshots:"
echo ""
echo "Available demos:"
echo "  • demo-quick.gif        - Quick overview"
echo "  • demo-basic.gif        - Basic walkthrough"
echo "  • demo-themes.gif       - Theme showcase"
echo "  • demo-workflow.gif     - Complete workflow"
echo "  • demo-examples.gif     - Code examples"
echo "  • demo-build.gif        - Build process"
echo "  • demo-architecture.gif - TEA explanation"
echo "  • demo-features.gif     - Feature showcase"
echo "  • demo-storage.gif      - Storage & persistence"
echo ""
echo "Usage in README:"
echo "  ![Demo](examples/vhs/demo-basic.gif)"
echo ""
