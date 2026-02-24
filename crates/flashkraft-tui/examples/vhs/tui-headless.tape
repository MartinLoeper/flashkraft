# VHS tape for FlashKraft TUI — Headless State Machine Demo
# Drives every screen transition and helper without a terminal or hardware.
# Run: vhs crates/flashkraft-tui/examples/vhs/tui-headless.tape

Output crates/flashkraft-tui/examples/vhs/generated/tui-headless.gif

Set Theme "Catppuccin Mocha"
Set Width 1400
Set Height 900
Set FontSize 14
Set PlaybackSpeed 1.5

Type "cargo run -p flashkraft-tui --example headless_demo"
Enter
Sleep 8s
