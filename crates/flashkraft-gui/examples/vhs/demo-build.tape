# VHS tape for FlashKraft GUI — Build & Verification
# Run: vhs crates/flashkraft-gui/examples/vhs/demo-build.tape

Output crates/flashkraft-gui/examples/vhs/generated/demo-build.gif
Set Theme "Catppuccin Mocha"
Set Width 1400
Set Height 900
Set FontSize 14
Set PlaybackSpeed 1.5

Type "rustc --version && cargo --version"
Enter
Sleep 1s

Type "ls -la crates/"
Enter
Sleep 1s

Type "cargo check -p flashkraft-gui -q 2>&1 | tail -5"
Enter
Sleep 20s

Type "cargo test -p flashkraft-gui --lib 2>&1 | tail -20"
Enter
Sleep 30s

Type "cargo build -p flashkraft-gui --release -q && echo 'Build OK'"
Enter
Sleep 60s

Type "ls -lh target/release/flashkraft"
Enter
Sleep 1s

Type "cargo doc -p flashkraft-gui --no-deps -q 2>&1 | tail -3 && echo 'Docs OK'"
Enter
Sleep 20s

Sleep 2s
