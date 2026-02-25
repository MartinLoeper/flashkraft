# VHS tape — FlashKraft TUI · Theme Switcher Demo
# Showcases the file-explorer theme switcher: cycling themes with t/[ and the
# theme-picker side panel toggled with T.
#
# Prerequisites:
#   cargo build -p flashkraft-tui --example theme_demo
#
# Run:
#   vhs crates/flashkraft-tui/examples/vhs/theme-switcher.tape

Output crates/flashkraft-tui/examples/vhs/generated/theme-switcher.gif

Set Theme "Catppuccin Mocha"
Set Width 1400
Set Height 900
Set FontSize 14
Set PlaybackSpeed 1.0

# ── Launch ────────────────────────────────────────────────────────────────────
Type "cargo run -p flashkraft-tui --example theme_demo"
Enter

# Wait for the file explorer to open with the theme panel visible
Sleep 5s

# ── Cycle forward through themes ──────────────────────────────────────────────
Type "t"
Sleep 600ms
Type "t"
Sleep 600ms
Type "t"
Sleep 600ms
Type "t"
Sleep 600ms
Type "t"
Sleep 600ms
Type "t"
Sleep 600ms
Type "t"
Sleep 600ms
Type "t"
Sleep 800ms

# ── Cycle backward a few steps ────────────────────────────────────────────────
Type "["
Sleep 600ms
Type "["
Sleep 600ms
Type "["
Sleep 800ms

# ── Continue forward through more themes ──────────────────────────────────────
Type "t"
Sleep 600ms
Type "t"
Sleep 600ms
Type "t"
Sleep 600ms
Type "t"
Sleep 600ms
Type "t"
Sleep 800ms

# ── Hide the theme panel and navigate the explorer ────────────────────────────
Type "T"
Sleep 1s

Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "k"
Sleep 500ms

# ── Cycle a couple more themes without the panel ─────────────────────────────
Type "t"
Sleep 600ms
Type "t"
Sleep 800ms

# ── Re-open the theme panel to show the final selection ───────────────────────
Type "T"
Sleep 1500ms

# ── Hold on the final state so viewers can read the theme panel ───────────────
Sleep 2s

# ── Quit ─────────────────────────────────────────────────────────────────────
Ctrl+Q
Sleep 500ms
