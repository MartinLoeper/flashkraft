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

# Example 1: Basic Usage
Type "# Example 1: Basic Usage - Counter with TEA"
Enter
Sleep 1s
Type "# Model-Message-Update-View pattern"
Enter
Sleep 2s

Type "cargo run --example basic_usage"
Enter
Sleep 5s
Screenshot examples/vhs/screenshots/example-01-basic-usage.png

# Let it run
Sleep 3s
Screenshot examples/vhs/screenshots/example-01-running.png

# Exit
Ctrl+C
Sleep 1s

# Example 2: Custom Theme
Type "# Example 2: Custom Themes - Dynamic Switching"
Enter
Sleep 1s
Type "# 21 beautiful themes to choose from"
Enter
Sleep 2s

Type "cargo run --example custom_theme"
Enter
Sleep 5s
Screenshot examples/vhs/screenshots/example-02-theme-dark.png

# Show theme switching
Sleep 3s
Screenshot examples/vhs/screenshots/example-02-theme-dracula.png

Sleep 3s
Screenshot examples/vhs/screenshots/example-02-theme-nord.png

# Exit
Ctrl+C
Sleep 1s

# Summary
Type "# Examples Summary"
Enter
Sleep 1s
Enter
Type "✓ basic_usage       - Complete FlashKraft app"
Enter
Type "✓ custom_theme      - Dynamic theme switching"
Enter
Sleep 1s
Enter
Type "# Features demonstrated:"
Enter
Type "  - Image file selection"
Enter
Type "  - Device detection"
Enter
Type "  - Flash progress tracking"
Enter
Type "  - 21 beautiful themes"
Enter
Type "  - The Elm Architecture"
Enter
Sleep 3s

Type "# Run any example with:"
Enter
Type "cargo run --example <name>"
Enter
Sleep 2s
Enter
Type "# Learn more in examples/README.md"
Enter
Sleep 3s
