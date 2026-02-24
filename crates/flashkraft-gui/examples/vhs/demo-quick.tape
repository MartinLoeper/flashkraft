# VHS tape for FlashKraft GUI — Quick Start
# Run: vhs crates/flashkraft-gui/examples/vhs/demo-quick.tape

Output crates/flashkraft-gui/examples/vhs/generated/demo-quick.gif
Set Theme "Catppuccin Mocha"
Set Width 1400
Set Height 900
Set FontSize 14
Set PlaybackSpeed 1.5

Type "rustc --version && cargo --version"
Enter
Sleep 1s

Type "cargo build -p flashkraft-gui -q && echo 'Build complete'"
Enter
Sleep 30s

Type "ls -lh target/debug/flashkraft"
Enter
Sleep 1s

Type "cargo run -p flashkraft-gui 2>/dev/null"
Enter
Sleep 5s

Ctrl+C
Sleep 1s
