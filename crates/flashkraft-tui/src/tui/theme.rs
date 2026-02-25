//! TUI Application Colour Palette
//!
//! This module bridges the [`tui_file_explorer::Theme`] preset catalogue with
//! the colour needs of the FlashKraft TUI.  Every named preset from the file
//! explorer is represented here as a [`TuiPalette`] that additionally carries
//! a background colour (`bg`) plus semantic `warn` and `err` colours that are
//! not part of the explorer's theme model.
//!
//! # Usage
//!
//! ```ignore
//! let themes = all_app_themes();           // Vec<(String, TuiPalette)>
//! let pal    = &themes[idx].1;
//! // then pass `pal` into every render_* function
//! ```

use ratatui::style::Color;
use tui_file_explorer::Theme;

// ── Palette struct ────────────────────────────────────────────────────────────

/// A complete colour palette for the FlashKraft TUI.
///
/// Fields map directly to the semantic roles used throughout `ui.rs`.
#[derive(Debug, Clone)]
pub struct TuiPalette {
    /// Brand / primary accent (titles, active elements).
    pub brand: Color,
    /// Secondary accent (borders, highlights).
    pub accent: Color,
    /// Positive / success state.
    pub success: Color,
    /// Warning / caution state.
    pub warn: Color,
    /// Error / destructive state.
    pub err: Color,
    /// Dimmed / secondary text.
    pub dim: Color,
    /// Default foreground.
    pub fg: Color,
    /// Terminal background fill.
    pub bg: Color,
    /// Selected-row background (list highlight).
    pub sel_bg: Color,
    /// Directory names in the file explorer.
    pub dir: Color,
}

impl Default for TuiPalette {
    /// The original FlashKraft palette — orange brand, sky-blue accent.
    fn default() -> Self {
        Self {
            brand: Color::Rgb(255, 100, 30),
            accent: Color::Rgb(80, 200, 255),
            success: Color::Rgb(80, 220, 120),
            warn: Color::Rgb(255, 200, 50),
            err: Color::Rgb(255, 80, 80),
            dim: Color::Rgb(120, 120, 130),
            fg: Color::White,
            bg: Color::Rgb(18, 18, 26),
            sel_bg: Color::Rgb(40, 60, 80),
            dir: Color::Rgb(255, 210, 80),
        }
    }
}

// ── Catalogue ─────────────────────────────────────────────────────────────────

/// Build the full list of named app themes.
///
/// The order mirrors [`tui_file_explorer::Theme::all_presets`] so that
/// `explorer_theme_idx` can serve as a shared index into both lists.
pub fn all_app_themes() -> Vec<(String, TuiPalette)> {
    Theme::all_presets()
        .into_iter()
        .map(|(name, _, t)| (name.to_string(), palette_from_preset(name, &t)))
        .collect()
}

// ── Internal mapping ──────────────────────────────────────────────────────────

/// Derive a [`TuiPalette`] from a file-explorer [`Theme`] preset.
///
/// The explorer theme already provides `brand`, `accent`, `success`, `dim`,
/// `fg`, `sel_bg`, and `dir`.  We add `bg`, `warn`, and `err` from a
/// hard-coded per-preset table that matches the visual intent of each scheme.
fn palette_from_preset(name: &str, t: &Theme) -> TuiPalette {
    let (bg, warn, err) = extras(name);
    TuiPalette {
        brand: t.brand,
        accent: t.accent,
        success: t.success,
        warn,
        err,
        dim: t.dim,
        fg: t.fg,
        bg,
        sel_bg: t.sel_bg,
        dir: t.dir,
    }
}

/// Per-theme background, warn, and error colours.
///
/// Returns `(bg, warn, err)`.
fn extras(name: &str) -> (Color, Color, Color) {
    match name {
        // ── Built-in ─────────────────────────────────────────────────────────
        "Default" => (
            Color::Rgb(18, 18, 26),
            Color::Rgb(255, 200, 50),
            Color::Rgb(255, 80, 80),
        ),

        // ── Decorative ────────────────────────────────────────────────────────
        "Grape" => (
            Color::Rgb(18, 12, 30),
            Color::Rgb(210, 170, 255),
            Color::Rgb(255, 80, 150),
        ),
        "Ocean" => (
            Color::Rgb(0, 20, 35),
            Color::Rgb(255, 220, 80),
            Color::Rgb(255, 100, 100),
        ),
        "Sunset" => (
            Color::Rgb(22, 8, 6),
            Color::Rgb(255, 230, 80),
            Color::Rgb(255, 50, 50),
        ),
        "Forest" => (
            Color::Rgb(8, 18, 8),
            Color::Rgb(220, 200, 80),
            Color::Rgb(210, 80, 80),
        ),
        "Rose" => (
            Color::Rgb(28, 6, 16),
            Color::Rgb(255, 220, 180),
            Color::Rgb(220, 60, 100),
        ),
        "Mono" => (
            Color::Rgb(8, 8, 10),
            Color::Rgb(200, 200, 200),
            Color::Rgb(160, 160, 160),
        ),
        "Neon" => (
            Color::Rgb(6, 0, 14),
            Color::Rgb(255, 220, 0),
            Color::Rgb(255, 30, 80),
        ),

        // ── Editor / terminal presets ─────────────────────────────────────────
        "Dracula" => (
            Color::Rgb(40, 42, 54),
            Color::Rgb(241, 250, 140),
            Color::Rgb(255, 85, 85),
        ),
        "Nord" => (
            Color::Rgb(29, 35, 42),
            Color::Rgb(235, 203, 139),
            Color::Rgb(191, 97, 106),
        ),
        "Solarized Dark" => (
            Color::Rgb(0, 43, 54),
            Color::Rgb(181, 137, 0),
            Color::Rgb(220, 50, 47),
        ),
        "Solarized Light" => (
            Color::Rgb(253, 246, 227),
            Color::Rgb(181, 137, 0),
            Color::Rgb(220, 50, 47),
        ),
        "Gruvbox Dark" => (
            Color::Rgb(29, 28, 27),
            Color::Rgb(250, 189, 47),
            Color::Rgb(251, 73, 52),
        ),
        "Gruvbox Light" => (
            Color::Rgb(251, 241, 199),
            Color::Rgb(215, 153, 33),
            Color::Rgb(214, 93, 14),
        ),
        "Catppuccin Latte" => (
            Color::Rgb(239, 241, 245),
            Color::Rgb(223, 142, 29),
            Color::Rgb(210, 15, 57),
        ),
        "Catppuccin Frappé" => (
            Color::Rgb(48, 52, 70),
            Color::Rgb(229, 200, 144),
            Color::Rgb(231, 130, 132),
        ),
        "Catppuccin Macchiato" => (
            Color::Rgb(36, 39, 58),
            Color::Rgb(238, 212, 159),
            Color::Rgb(237, 135, 150),
        ),
        "Catppuccin Mocha" => (
            Color::Rgb(30, 30, 46),
            Color::Rgb(249, 226, 175),
            Color::Rgb(243, 139, 168),
        ),
        "Tokyo Night" => (
            Color::Rgb(26, 27, 38),
            Color::Rgb(224, 175, 104),
            Color::Rgb(247, 118, 142),
        ),
        "Tokyo Night Storm" => (
            Color::Rgb(36, 40, 59),
            Color::Rgb(224, 175, 104),
            Color::Rgb(247, 118, 142),
        ),
        "Tokyo Night Light" => (
            Color::Rgb(213, 214, 219),
            Color::Rgb(140, 108, 62),
            Color::Rgb(210, 15, 57),
        ),
        "Kanagawa Wave" => (
            Color::Rgb(22, 22, 30),
            Color::Rgb(220, 165, 97),
            Color::Rgb(210, 126, 153),
        ),
        "Kanagawa Dragon" => (
            Color::Rgb(20, 20, 20),
            Color::Rgb(200, 170, 109),
            Color::Rgb(210, 126, 153),
        ),
        "Kanagawa Lotus" => (
            Color::Rgb(246, 243, 228),
            Color::Rgb(119, 113, 63),
            Color::Rgb(192, 71, 71),
        ),
        "Moonfly" => (
            Color::Rgb(8, 8, 8),
            Color::Rgb(226, 164, 120),
            Color::Rgb(255, 115, 131),
        ),
        "Nightfly" => (
            Color::Rgb(1, 22, 38),
            Color::Rgb(243, 218, 11),
            Color::Rgb(252, 87, 73),
        ),
        "Oxocarbon" => (
            Color::Rgb(22, 22, 22),
            Color::Rgb(250, 204, 55),
            Color::Rgb(255, 97, 101),
        ),

        // ── Fallback — reuse the default FlashKraft palette ───────────────────
        _ => (
            Color::Rgb(18, 18, 26),
            Color::Rgb(255, 200, 50),
            Color::Rgb(255, 80, 80),
        ),
    }
}
