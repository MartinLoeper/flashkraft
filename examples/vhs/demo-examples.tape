# VHS tape for FlashKraft - Examples Demo
# This demonstrates running the various examples

Output examples/vhs/demo-examples.gif
Set FontSize 16
Set Width 1200
Set Height 800
Set Theme "TokyoNight"
Set PlaybackSpeed 0.9

# Title card
Type "# FlashKraft Examples - The Elm Architecture"
Enter
Sleep 2s
Enter
Type "Demonstrating Rust examples with Iced framework"
Enter
Sleep 2s
Clear

# Example 1: Basic Usage
Type "# Example 1: Basic Usage - Counter with TEA"
Enter
Sleep 1s
Type "# Model-Message-Update-View pattern"
Enter
Sleep 2s
Clear

Type "cargo run --example basic_usage"
Enter
Sleep 5s
Screenshot examples/vhs/screenshots/example-01-basic-usage.png

# Let it run
Sleep 3s
Screenshot examples/vhs/screenshots/example-01-basic-counter.png

# Exit
Ctrl+C
Sleep 1s
Clear

# Example 2: State Machine
Type "# Example 2: State Machine - Workflow States"
Enter
Sleep 1s
Type "# Using enums to model distinct application states"
Enter
Sleep 2s
Clear

Type "cargo run --example state_machine"
Enter
Sleep 5s
Screenshot examples/vhs/screenshots/example-02-state-machine-idle.png

# Let it run to show different states
Sleep 3s
Screenshot examples/vhs/screenshots/example-02-state-machine-selected.png

Sleep 3s
Screenshot examples/vhs/screenshots/example-02-state-machine-processing.png

# Exit
Ctrl+C
Sleep 1s
Clear

# Example 3: Custom Theme
Type "# Example 3: Custom Themes - Dynamic Switching"
Enter
Sleep 1s
Type "# 21 beautiful themes to choose from"
Enter
Sleep 2s
Clear

Type "cargo run --example custom_theme"
Enter
Sleep 5s
Screenshot examples/vhs/screenshots/example-03-theme-dark.png

# Show theme switching
Sleep 3s
Screenshot examples/vhs/screenshots/example-03-theme-dracula.png

Sleep 3s
Screenshot examples/vhs/screenshots/example-03-theme-nord.png

# Exit
Ctrl+C
Sleep 1s
Clear

# Summary
Type "# Examples Summary"
Enter
Sleep 1s
Enter
Type "✓ basic_usage       - Core TEA concepts"
Enter
Type "✓ state_machine     - State modeling with enums"
Enter
Type "✓ custom_theme      - Dynamic theme switching"
Enter
Sleep 1s
Enter
Type "# More examples available:"
Enter
Type "  - async_commands.rs"
Enter
Type "  - subscription_progress.rs"
Enter
Type "  - file_dialog.rs"
Enter
Type "  - drive_detection.rs"
Enter
Type "  - layout_composition.rs"
Enter
Sleep 3s
Clear

Type "# Run any example with:"
Enter
Type "cargo run --example <name>"
Enter
Sleep 2s
Enter
Type "# Learn more in examples/README.md"
Enter
Sleep 3s
