# VHS tape for FlashKraft TUI — Full Demo
# Run: vhs crates/flashkraft-tui/examples/vhs/tui-demo.tape

Output crates/flashkraft-tui/examples/vhs/generated/tui-demo.gif

Set Theme "Catppuccin Mocha"
Set Width 1400
Set Height 900
Set FontSize 14
Set PlaybackSpeed 1.5

Type "dd if=/dev/zero of=/tmp/flashkraft-demo.iso bs=1M count=1 2>/dev/null && cargo run -p flashkraft-tui --example tui_demo"
Enter
Sleep 5s

# SelectImage landing screen
Sleep 1s

# Enter editing mode and type image path
Type "i"
Sleep 500ms

Type "/tmp/flashkraft-demo.iso"
Sleep 500ms

# Open the file browser with Tab
Tab
Sleep 1s

# Navigate down
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "k"
Sleep 300ms

# Page through the listing
PageDown
Sleep 500ms
PageUp
Sleep 500ms

# Ascend to parent
Backspace
Sleep 800ms

# Dismiss browser
Escape
Sleep 800ms

# Confirm the image path → SelectDrive
Enter
Sleep 1s

# Refresh and navigate the drive list
Type "r"
Sleep 2s

Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "k"
Sleep 400ms

# Attempt to confirm first drive (no-op if list empty)
Enter
Sleep 1s

# DriveInfo → ConfirmFlash (ignored if not on DriveInfo)
Type "f"
Sleep 1s

# Cancel back — never write to real hardware
Type "b"
Sleep 500ms
Type "b"
Sleep 500ms
Type "b"
Sleep 500ms

# Back on SelectImage — quit
Escape
Sleep 1s
