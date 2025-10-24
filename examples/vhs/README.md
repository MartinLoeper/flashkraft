# FlashKraft VHS Examples

This directory contains VHS tape files for generating animated demos and screenshots of FlashKraft.

[VHS](https://github.com/charmbracelet/vhs) allows you to write test scripts for your CLI and automatically generate terminal GIFs and screenshots.

## Prerequisites

Install VHS:

```bash
# macOS or Linux
brew install vhs

# Or using go
go install github.com/charmbracelet/vhs@latest
```

## Available Tapes

### 1. `demo-basic.tape`
Basic demonstration of FlashKraft's main interface and features.

**Generates:**
- `demo-basic.gif` - Animated walkthrough
- Screenshots in `screenshots/` directory

**Run:**
```bash
vhs vhs/demo-basic.tape
```

### 2. `demo-themes.tape`
Showcases all 21 available themes in FlashKraft.

**Generates:**
- `demo-themes.gif` - Theme switching animation
- Individual screenshots for each theme

**Run:**
```bash
vhs vhs/demo-themes.tape
```

### 3. `demo-workflow.tape`
Complete workflow demonstration from image selection to successful flash.

**Generates:**
- `demo-workflow.gif` - Full workflow animation
- Step-by-step screenshots

**Run:**
```bash
vhs vhs/demo-workflow.tape
```

## Generating All Demos

Run all tapes at once:

```bash
# From project root
cd flashkraft
vhs vhs/demo-basic.tape
vhs vhs/demo-themes.tape
vhs vhs/demo-workflow.tape
```

Or use a script:

```bash
for tape in vhs/*.tape; do
    echo "Processing $tape..."
    vhs "$tape"
done
```

## Output

Generated files will be placed in:
- `vhs/*.gif` - Animated GIFs
- `vhs/screenshots/*.png` - Individual screenshots

## Customization

You can customize the tapes by editing the `.tape` files:

```tape
# Change output settings
Set FontSize 18
Set Width 1400
Set Height 900
Set Theme "Nord"
Set PlaybackSpeed 1.0

# Adjust timing
Sleep 2s

# Take screenshots
Screenshot path/to/output.png

# Simulate typing
Type "cargo run"
Enter
```

## VHS Commands Used

Common commands in our tapes:

- `Output <file>` - Set output file path
- `Set <setting> <value>` - Configure appearance
- `Type <text>` - Simulate typing
- `Enter` - Press Enter key
- `Sleep <duration>` - Wait (e.g., 2s, 500ms)
- `Screenshot <file>` - Capture screenshot
- `Ctrl+C` - Send interrupt signal
- `Clear` - Clear the terminal

## Screenshot Organization

Screenshots are organized by demo type:

```
vhs/screenshots/
├── 01-initial-state.png
├── 02-with-theme-selector.png
├── 03-image-selected.png
├── 04-device-selection.png
├── 05-ready-to-flash.png
├── theme-01-dark.png
├── theme-02-light.png
├── theme-03-dracula.png
├── workflow-01-initial.png
├── workflow-02-before-image-select.png
└── ...
```

## Using in Documentation

Add generated GIFs to README or documentation:

```markdown
![FlashKraft Demo](vhs/demo-basic.gif)

## Theme Support
![Theme Switching](vhs/demo-themes.gif)

## Complete Workflow
![Full Workflow](vhs/demo-workflow.gif)
```

## Tips

1. **Recording Quality**
   - Use higher resolution for better screenshots
   - Adjust `PlaybackSpeed` for smoother animations
   - Increase `FontSize` for better readability

2. **File Size**
   - GIFs can be large - consider compression
   - Use `gifsicle` to optimize: `gifsicle -O3 input.gif -o output.gif`

3. **Interactive UI**
   - VHS works best with CLI applications
   - For GUI apps like FlashKraft, you may need to:
     - Run the app in one terminal
     - Use VHS to show the terminal interaction
     - Manually capture UI screenshots

## Manual Screenshots

Since FlashKraft is a GUI application, you may want to:

1. Run the app: `cargo run`
2. Manually navigate through the UI
3. Take screenshots at each step
4. Place them in `vhs/screenshots/`
5. Reference them in documentation

Or use a screen recording tool like:
- **Linux**: SimpleScreenRecorder, Peek, Kazam
- **macOS**: QuickTime, Kap
- **Windows**: OBS Studio

## Contributing

When adding new VHS tapes:

1. Create a descriptive `.tape` file
2. Follow naming convention: `demo-<feature>.tape`
3. Include comments explaining what's being demonstrated
4. Document in this README
5. Organize screenshots logically

## Examples from Other Projects

Learn more VHS usage from:
- [VHS Examples](https://github.com/charmbracelet/vhs/tree/main/examples)
- [Charm Gallery](https://charm.sh/)

---

**Note:** Since FlashKraft is a GUI application built with Iced, VHS tapes are best used for:
- Showing the terminal build/run process
- Documenting CLI interactions
- Creating title cards and transitions

For actual UI demonstrations, consider:
- Manual screenshots
- Screen recording software
- Automated UI testing with screenshot capture