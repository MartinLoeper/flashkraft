# Icons and Animations

This document describes the icon system and animated progress lines implemented in FlashKraft.

## Overview

FlashKraft uses **Bootstrap Icons** for a professional, consistent visual design throughout the application. The icons are complemented by **animated progress lines** that provide visual feedback on workflow progression.

## Icon System

### Bootstrap Icons Integration

FlashKraft uses the Bootstrap Icons font (v1.11.3) loaded via the `iced_fonts` crate:

```rust
// In main.rs
const BOOTSTRAP_ICONS: &[u8] = include_bytes!("../fonts/bootstrap-icons.woff2");

FlashKraft::run(Settings {
    fonts: vec![BOOTSTRAP_ICONS.into()],
    // ...
})
```

### Font Configuration

Icons are rendered using a specific font configuration:

```rust
let icon_font = iced::Font {
    family: iced::font::Family::Name("bootstrap-icons"),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};
```

### Icon Constants

All icons are defined as Unicode constants in `view.rs`:

```rust
const ICON_FILE: &str = "\u{F377}";     // file-earmark
const ICON_DEVICE: &str = "\u{F4A7}";   // device-ssd
const ICON_FLASH: &str = "\u{F3A4}";    // lightning-fill
const ICON_CHECK: &str = "\u{F272}";    // check-circle-fill
const ICON_PLUS: &str = "\u{F5D4}";     // plus-circle
const ICON_CLOSE: &str = "\u{F659}";    // x-lg
const ICON_WARNING: &str = "\u{F33A}";  // exclamation-triangle-fill
```

### Icon Usage Throughout the App

#### Header
```rust
text(ICON_FLASH)
    .font(icon_font)
    .size(32)
```
- Lightning bolt icon next to "FlashKraft" title
- Size: 32px
- Color: Theme default (white on dark theme)

#### Step Indicator

**Step 1 - Select Image**
```rust
text(if has_image { ICON_CHECK } else { ICON_FILE })
    .font(icon_font)
    .size(40)
    .style(if has_image {
        Color::from_rgb(0.3, 0.8, 0.3)  // Green when complete
    } else {
        Color::WHITE                     // White when pending
    })
```
- File icon when pending
- Check-circle icon when image selected
- Size: 40px

**Step 2 - Select Target**
```rust
text(if has_target { ICON_CHECK } else { ICON_DEVICE })
    .font(icon_font)
    .size(40)
    .style(if has_target {
        Color::from_rgb(0.3, 0.8, 0.3)  // Green when complete
    } else {
        Color::WHITE                     // White when pending
    })
```
- Device/SSD icon when pending
- Check-circle icon when target selected
- Size: 40px

**Step 3 - Flash**
```rust
text(if is_ready { ICON_CHECK } else { ICON_FLASH })
    .font(icon_font)
    .size(40)
    .style(if is_ready {
        Color::from_rgb(0.3, 0.8, 0.3)  // Green when ready
    } else {
        Color::WHITE                     // White when not ready
    })
```
- Lightning icon when not ready
- Check-circle icon when ready to flash
- Size: 40px

#### Selection Panels

**Image Panel**
```rust
// When no image selected
text(ICON_PLUS)
    .font(icon_font)
    .size(50)

// When image selected
text(ICON_CHECK)
    .font(icon_font)
    .size(50)
    .style(Color::from_rgb(0.3, 0.8, 0.3))
```

**Target Panel**
```rust
// When no target selected
text(ICON_DEVICE)
    .font(icon_font)
    .size(50)

// When target selected
text(ICON_CHECK)
    .font(icon_font)
    .size(50)
    .style(Color::from_rgb(0.3, 0.8, 0.3))
```

**Flash Panel**
```rust
text(ICON_FLASH)
    .font(icon_font)
    .size(50)
    .style(if is_ready {
        Color::from_rgb(0.3, 0.8, 0.3)
    } else {
        Color::WHITE
    })
```

#### Device Selector Modal

**Close Button**
```rust
button(text(ICON_CLOSE).font(icon_font).size(20))
    .on_press(Message::CloseDeviceSelection)
```
- X icon for closing modal
- Size: 20px

**Warning Indicators**
```rust
text(ICON_WARNING)
    .font(icon_font)
    .style(Color::from_rgb(1.0, 0.6, 0.0))  // Orange
    .size(14)
```
- Triangle warning icon for system drives
- Size: 14px
- Color: Orange (#FF9900)

#### Status Views

**Flashing Progress**
```rust
text(ICON_FLASH)
    .font(icon_font)
    .size(80)
    .style(Color::from_rgb(0.3, 0.8, 0.3))
```

**Error State**
```rust
text(ICON_WARNING)
    .font(icon_font)
    .size(80)
    .style(Color::from_rgb(1.0, 0.3, 0.3))  // Red
```

**Complete State**
```rust
text(ICON_CHECK)
    .font(icon_font)
    .size(80)
    .style(Color::from_rgb(0.3, 0.8, 0.3))
```

## Color Scheme

### Icon Colors

| State | Color | RGB | Usage |
|-------|-------|-----|-------|
| **Pending** | White | `(1.0, 1.0, 1.0)` | Default state, not yet completed |
| **Complete** | Green | `(0.3, 0.8, 0.3)` | #4DCC4D - Completed steps |
| **Warning** | Orange | `(1.0, 0.6, 0.0)` | #FF9900 - System drives |
| **Error** | Red | `(1.0, 0.3, 0.3)` | #FF4D4D - Error states |

### Semantic Color Usage

- **Green (Success)**: Selected images, selected targets, completed steps, successful flash
- **Orange (Warning)**: System drives, potential risks
- **Red (Error)**: Flash errors, critical warnings
- **White (Neutral)**: Pending steps, default states

## Animated Progress Lines

### Implementation

Progress lines are drawn using Iced's Canvas API:

```rust
struct ProgressLine {
    color: Color,
    width: f32,
}

impl canvas::Program<Message> for ProgressLine {
    type State = ();

    fn draw(&self, ...) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let line = Path::line(
            iced::Point::new(0.0, bounds.height / 2.0),
            iced::Point::new(self.width, bounds.height / 2.0),
        );

        frame.stroke(
            &line,
            Stroke::default()
                .with_color(self.color)
                .with_width(4.0),
        );

        vec![frame.into_geometry()]
    }
}
```

### Line Specifications

- **Width**: 100px (fixed)
- **Height**: 4px (stroke width)
- **Position**: Centered vertically between step indicators
- **Color**: 
  - Green `(0.3, 0.8, 0.3)` when step is completed
  - Gray `(0.3, 0.3, 0.3)` when step is pending

### Workflow Progression

#### Line 1: Select Image → Select Target
- **State**: Pending (gray)
- **Condition**: No image selected
- **Transitions to green**: When `state.selected_image.is_some()`

#### Line 2: Select Target → Flash
- **State**: Pending (gray)
- **Condition**: No target selected
- **Transitions to green**: When `state.selected_target.is_some()`

### Visual Flow

```
┌─────────┐         ┌─────────┐         ┌─────────┐
│  FILE   │ ─────── │ DEVICE  │ ─────── │  FLASH  │
│  ICON   │  gray   │  ICON   │  gray   │  ICON   │
└─────────┘         └─────────┘         └─────────┘
     ↓                    ↓                   ↓
┌─────────┐         ┌─────────┐         ┌─────────┐
│  CHECK  │ ═══════ │ DEVICE  │ ─────── │  FLASH  │
│  ICON   │  green  │  ICON   │  gray   │  ICON   │
└─────────┘         └─────────┘         └─────────┘
     ↓                    ↓                   ↓
┌─────────┐         ┌─────────┐         ┌─────────┐
│  CHECK  │ ═══════ │  CHECK  │ ═══════ │  FLASH  │
│  ICON   │  green  │  ICON   │  green  │  ICON   │
└─────────┘         └─────────┘         └─────────┘
     ↓                    ↓                   ↓
┌─────────┐         ┌─────────┐         ┌─────────┐
│  CHECK  │ ═══════ │  CHECK  │ ═══════ │  CHECK  │
│  ICON   │  green  │  ICON   │  green  │  ICON   │
└─────────┘         └─────────┘         └─────────┘
```

## Performance Considerations

### Font Loading

- **Font Size**: 128KB (bootstrap-icons.woff2)
- **Loading**: Embedded at compile time with `include_bytes!`
- **Memory**: Minimal overhead, fonts are efficient
- **Rendering**: GPU-accelerated via Iced

### Canvas Rendering

- **Performance**: Excellent, hardware-accelerated
- **Frame Rate**: Smooth 60fps
- **Memory**: Minimal, simple geometry
- **Scalability**: Lines are resolution-independent

### Best Practices

1. **Reuse Font Configuration**: Create `icon_font` once per view function
2. **Const Icons**: Use const strings for icon unicode points
3. **Conditional Styling**: Use `if` expressions for dynamic colors
4. **Canvas Caching**: Future enhancement for complex animations

## Future Enhancements

### Planned Improvements

1. **Animated Line Transitions**
   - Smooth width animation from 0 to 100px
   - Easing functions for natural motion
   - Duration: 300-500ms

2. **Icon Animations**
   - Fade transitions between icons
   - Scale animations on state change
   - Rotation for loading states

3. **Progress Animation**
   - Pulsing effects for active steps
   - Glow effects for ready states
   - Ripple effects on interactions

4. **Custom Themes**
   - Light theme color variants
   - High contrast mode
   - Colorblind-friendly palettes

5. **More Icons**
   - Settings gear icon
   - Refresh/reload icon
   - Information icon
   - Help icon

### Implementation Notes

For animated line width transitions:

```rust
// Future enhancement
struct AnimatedProgressLine {
    color: Color,
    progress: f32,  // 0.0 to 1.0
    max_width: f32,
}

impl AnimatedProgressLine {
    fn draw(&self, ...) -> Vec<canvas::Geometry> {
        let animated_width = self.max_width * self.progress;
        // Draw line with animated_width
    }
}
```

## Troubleshooting

### Icons Not Displaying

**Problem**: Icons show as squares or missing glyphs

**Solutions**:
1. Verify font file exists: `fonts/bootstrap-icons.woff2`
2. Check font is loaded in `main.rs`
3. Ensure correct unicode points in constants
4. Verify `iced_fonts` dependency is included

### Lines Not Rendering

**Problem**: Progress lines don't appear

**Solutions**:
1. Verify `canvas` feature is enabled in Cargo.toml
2. Check `iced::widget::canvas` import
3. Ensure Canvas widget has proper size bounds
4. Verify stroke width > 0

### Color Issues

**Problem**: Icons/lines have wrong colors

**Solutions**:
1. Check color RGB values are in 0.0-1.0 range
2. Verify conditional color logic
3. Test with different themes
4. Check alpha channel if using RGBA

## Resources

- **Bootstrap Icons**: https://icons.getbootstrap.com/
- **Iced Canvas**: https://docs.rs/iced/latest/iced/widget/canvas/
- **Color Theory**: Choose colors with proper contrast ratios
- **Unicode Tables**: Verify icon codepoints match Bootstrap Icons font

## Conclusion

The icon and animation system in FlashKraft provides:

- **Professional Appearance**: Consistent, high-quality icons
- **Clear Visual Feedback**: State changes are immediately visible
- **Performance**: Efficient rendering with minimal overhead
- **Extensibility**: Easy to add new icons and animations
- **Maintainability**: Centralized constants and reusable patterns

The system balances visual appeal with performance, providing a polished user experience without sacrificing functionality.