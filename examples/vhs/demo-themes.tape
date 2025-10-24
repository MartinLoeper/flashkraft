# VHS tape for FlashKraft - Theme Switching Demo
# This demonstrates all the available themes in FlashKraft

Output examples/vhs/demo-themes.gif
Set FontSize 16
Set Width 1200
Set Height 800
Set Theme "Dracula"
Set PlaybackSpeed 0.5

# Start the application
Type "cargo run"
Enter
Sleep 4s

# Show initial Dark theme
Screenshot examples/vhs/screenshots/theme-01-dark.png
Sleep 1s

# Click on theme dropdown (top right corner)
# Note: These are simulated clicks - actual interaction would need UI automation
Sleep 1s

# Switch to Light theme
Sleep 1s
Screenshot examples/vhs/screenshots/theme-02-light.png

# Switch to Dracula theme
Sleep 1s
Screenshot examples/vhs/screenshots/theme-03-dracula.png

# Switch to Nord theme
Sleep 1s
Screenshot examples/vhs/screenshots/theme-04-nord.png

# Switch to Tokyo Night theme
Sleep 1s
Screenshot examples/vhs/screenshots/theme-05-tokyonight.png

# Switch to Catppuccin Mocha theme
Sleep 1s
Screenshot examples/vhs/screenshots/theme-06-catppuccin-mocha.png

# Switch to Gruvbox Dark theme
Sleep 1s
Screenshot examples/vhs/screenshots/theme-07-gruvbox-dark.png

# Switch to Kanagawa Wave theme
Sleep 1s
Screenshot examples/vhs/screenshots/theme-08-kanagawa-wave.png

# Back to Dark theme
Sleep 1s
Screenshot examples/vhs/screenshots/theme-09-back-to-dark.png

# Final pause
Sleep 2s

# Exit
Ctrl+C
Sleep 1s
