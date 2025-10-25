# VHS tape for FlashKraft - Quick Demo
# A short demo for testing VHS setup and showing FlashKraft basics

Output examples/vhs/demo-quick.gif
Set FontSize 16
Set Width 1200
Set Height 800
Set Theme "Dracula"

# Title
Type "# FlashKraft - OS Image Writer"
Enter
Sleep 1s
Type "# Quick Demo"
Enter
Sleep 2s

# Show build command
Type "cargo build --release"
Enter
Sleep 3s

# Run the application
Type "cargo run"
Enter
Sleep 5s

# Let it run and show the UI
Sleep 3s
Screenshot examples/vhs/screenshots/quick-01-main-ui.png

# Wait a bit more
Sleep 2s
Screenshot examples/vhs/screenshots/quick-02-interface.png

# Final pause
Sleep 2s

# Exit gracefully
Ctrl+C
Sleep 1s

# Done
Type "Demo complete! ✓"
Enter
Sleep 1s
