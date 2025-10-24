#!/bin/bash
# Generate all FlashKraft VHS demos
# This script runs all VHS tapes and generates GIFs and screenshots

set -e

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
)

# Process each tape
for tape in "${TAPES[@]}"; do
    TAPE_FILE="examples/vhs/${tape}.tape"

    if [ -f "$TAPE_FILE" ]; then
        echo "─────────────────────────────────────────────"
        echo "Processing: ${tape}.tape"
        echo "─────────────────────────────────────────────"

        # Run VHS
        if vhs "$TAPE_FILE"; then
            echo "✓ Generated: examples/vhs/${tape}.gif"
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
echo "  📁 examples/vhs/"
ls -lh examples/vhs/*.gif 2>/dev/null || echo "    (no GIF files generated)"
echo ""
echo "  📁 examples/vhs/screenshots/"
ls -1 examples/vhs/screenshots/*.png 2>/dev/null | wc -l | xargs echo "    Screenshots:"
echo ""
echo "Available demos:"
echo "  • demo-quick.gif        - Quick overview"
echo "  • demo-basic.gif        - Basic walkthrough"
echo "  • demo-themes.gif       - Theme showcase"
echo "  • demo-workflow.gif     - Complete workflow"
echo "  • demo-examples.gif     - Code examples"
echo "  • demo-build.gif        - Build process"
echo "  • demo-architecture.gif - TEA explanation"
echo ""
echo "Usage in README:"
echo "  ![Demo](examples/vhs/demo-basic.gif)"
echo ""
