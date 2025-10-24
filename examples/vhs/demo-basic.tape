# VHS tape for FlashKraft - Basic Demo
# This demonstrates the basic workflow of FlashKraft

# Output settings
Output examples/vhs/demo-basic.gif
Set FontSize 16
Set Width 1200
Set Height 800
Set Theme "Dracula"

# Start the application
Type "cargo run"
Enter
Sleep 3s

# Show initial state
Sleep 2s
Screenshot examples/vhs/screenshots/01-initial-state.png

# Hover over Select Image button
Sleep 1s

# Move to theme selector (top right)
Sleep 1s
Screenshot examples/vhs/screenshots/02-with-theme-selector.png

# Select an image (simulated)
Sleep 1s
Screenshot examples/vhs/screenshots/03-image-selected.png

# Open device selection
Sleep 1s
Screenshot examples/vhs/screenshots/04-device-selection.png

# Select a target drive
Sleep 1s
Screenshot examples/vhs/screenshots/05-ready-to-flash.png

# Show ready to flash state
Sleep 2s
Screenshot examples/vhs/screenshots/06-flash-ready.png

# Final state
Sleep 1s

# Close
Ctrl+C
Sleep 1s
