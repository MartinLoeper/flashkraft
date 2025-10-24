# VHS tape for FlashKraft - Complete Workflow Demo
# This demonstrates the full workflow from start to finish

Output examples/vhs/demo-workflow.gif
Set FontSize 16
Set Width 1200
Set Height 800
Set Theme "Dracula"
Set PlaybackSpeed 0.8

# Title card
Type "# FlashKraft - Complete Workflow Demo"
Enter
Sleep 2s
Clear

# Start the application
Type "cargo run"
Enter
Sleep 4s

# Initial state - Show the clean UI
Sleep 2s
Screenshot examples/vhs/screenshots/workflow-01-initial.png

# Step 1: Select Image
Sleep 1s
Type@100ms "Clicking 'Flash from file' button..."
Sleep 1s
Screenshot examples/vhs/screenshots/workflow-02-before-image-select.png

# Simulate image selection dialog
Sleep 2s
Screenshot examples/vhs/screenshots/workflow-03-image-selected.png
Sleep 1s

# Step 2: Open Device Selection
Sleep 1s
Type@100ms "Opening device selection..."
Sleep 1s
Screenshot examples/vhs/screenshots/workflow-04-device-selection-open.png

# Show available drives
Sleep 2s
Screenshot examples/vhs/screenshots/workflow-05-drives-list.png

# Select a drive
Sleep 1s
Screenshot examples/vhs/screenshots/workflow-06-drive-selected.png

# Step 3: Ready to Flash
Sleep 1s
Screenshot examples/vhs/screenshots/workflow-07-ready-to-flash.png

# Click Flash button
Sleep 2s
Screenshot examples/vhs/screenshots/workflow-08-flash-button-ready.png

# Step 4: Flashing in Progress
Sleep 1s
Screenshot examples/vhs/screenshots/workflow-09-flashing-started.png

# Show progress at 25%
Sleep 2s
Screenshot examples/vhs/screenshots/workflow-10-progress-25.png

# Show progress at 50%
Sleep 2s
Screenshot examples/vhs/screenshots/workflow-11-progress-50.png

# Show progress at 75%
Sleep 2s
Screenshot examples/vhs/screenshots/workflow-12-progress-75.png

# Step 5: Flash Complete
Sleep 2s
Screenshot examples/vhs/screenshots/workflow-13-complete.png

# Show success message
Sleep 3s
Screenshot examples/vhs/screenshots/workflow-14-success.png

# Reset and start over
Sleep 2s
Screenshot examples/vhs/screenshots/workflow-15-flash-another.png

# Final state
Sleep 2s

# Exit
Ctrl+C
Sleep 1s
