//! # ratatui-file-explorer
//!
//! A self-contained, reusable file-browser widget for [Ratatui].
//!
//! ## Design goals
//!
//! * **Zero flashkraft dependencies** — only `ratatui`, `crossterm`, and the
//!   standard library. Drop the module into any Ratatui project as-is.
//! * **Extractable** — the public surface (types + `render` function) is
//!   intentionally narrow so this module can be published as a stand-alone
//!   crate with no breaking changes.
//! * **Extension filtering** — callers pass a list of allowed extensions so
//!   that only relevant files are selectable (e.g. `["iso", "img"]`).
//! * **Keyboard-driven** — arrow keys / vim keys, `Enter` to descend or
//!   confirm, `Backspace` / `h` to ascend, `Esc` to dismiss.
//!
//! ## Typical usage
//!
//! ```no_run
//! use flashkraft_tui::file_explorer::{FileExplorer, ExplorerOutcome};
//! use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
//!
//! // Create once (e.g. in App::new)
//! let mut explorer = FileExplorer::new(
//!     dirs::home_dir().unwrap_or_default(),
//!     vec!["iso".into(), "img".into()],
//! );
//!
//! // In the render function (inside a ratatui Terminal::draw closure):
//! // flashkraft_tui::file_explorer::render(&mut explorer, frame, frame.area());
//!
//! // In the key-handler
//! # let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
//! match explorer.handle_key(key) {
//!     ExplorerOutcome::Selected(path) => { /* use path */ }
//!     ExplorerOutcome::Dismissed      => { /* close the explorer */ }
//!     _                               => {}
//! }
//! ```

use std::{
    fs,
    path::{Path, PathBuf},
};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Padding, Paragraph},
    Frame,
};

// ── Palette (mirrors the FlashKraft palette; override freely when extracting) ─

const C_BRAND: Color = Color::Rgb(255, 100, 30);
const C_ACCENT: Color = Color::Rgb(80, 200, 255);
const C_SUCCESS: Color = Color::Rgb(80, 220, 120);
const C_DIM: Color = Color::Rgb(120, 120, 130);
const C_FG: Color = Color::White;
const C_SEL_BG: Color = Color::Rgb(40, 60, 80);
const C_DIR: Color = Color::Rgb(255, 210, 80);
const C_MATCH: Color = Color::Rgb(80, 220, 120);

// ── Public types ──────────────────────────────────────────────────────────────

/// A single entry shown in the file-explorer list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FsEntry {
    /// Display name (filename only, not full path).
    pub name: String,
    /// Absolute path to the entry.
    pub path: PathBuf,
    /// `true` if the entry is a directory.
    pub is_dir: bool,
    /// File size in bytes (`None` for directories or when unavailable).
    pub size: Option<u64>,
    /// File extension in lower-case (empty string for directories / no ext).
    pub extension: String,
}

/// Outcome returned by [`FileExplorer::handle_key`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExplorerOutcome {
    /// The user confirmed a file selection — contains the chosen path.
    Selected(PathBuf),
    /// The user dismissed the explorer (Esc) without selecting anything.
    Dismissed,
    /// A key was consumed but produced no navigational outcome yet.
    Pending,
    /// The key was not recognised / consumed by the explorer.
    Unhandled,
}

/// State for the file-explorer widget.
///
/// Keep one instance in your application state and pass a mutable reference
/// to [`render`] and [`FileExplorer::handle_key`] on every frame / key event.
#[derive(Debug)]
pub struct FileExplorer {
    /// The directory currently being browsed.
    pub current_dir: PathBuf,
    /// Sorted list of visible entries (dirs first, then files).
    pub entries: Vec<FsEntry>,
    /// Index of the highlighted entry.
    pub cursor: usize,
    /// Index of the first visible entry (for scrolling).
    scroll_offset: usize,
    /// Only files whose extension is in this list are selectable.
    /// Directories are always shown and always navigable.
    /// An empty `Vec` means *all* files are selectable.
    pub extension_filter: Vec<String>,
    /// Whether to show dotfiles / hidden entries.
    pub show_hidden: bool,
    /// Human-readable status message (shown in the footer).
    status: String,
}

impl FileExplorer {
    // ── Construction ─────────────────────────────────────────────────────────

    /// Create a new explorer starting at `initial_dir`.
    ///
    /// `extension_filter` is a list of lower-case extensions *without* the
    /// leading dot (e.g. `vec!["iso".into(), "img".into()]`).
    /// Pass an empty `Vec` to allow all files.
    pub fn new(initial_dir: PathBuf, extension_filter: Vec<String>) -> Self {
        let mut explorer = Self {
            current_dir: initial_dir,
            entries: Vec::new(),
            cursor: 0,
            scroll_offset: 0,
            extension_filter,
            show_hidden: false,
            status: String::new(),
        };
        explorer.reload();
        explorer
    }

    /// Navigate to `path`, resetting cursor and scroll.
    pub fn navigate_to(&mut self, path: PathBuf) {
        self.current_dir = path;
        self.cursor = 0;
        self.scroll_offset = 0;
        self.reload();
    }

    // ── Key handling ─────────────────────────────────────────────────────────

    /// Process a single keyboard event and return the [`ExplorerOutcome`].
    ///
    /// Call this from your application's key-handling function and act on
    /// [`ExplorerOutcome::Selected`] / [`ExplorerOutcome::Dismissed`].
    pub fn handle_key(&mut self, key: KeyEvent) -> ExplorerOutcome {
        match key.code {
            // ── Dismiss ──────────────────────────────────────────────────────
            KeyCode::Esc => ExplorerOutcome::Dismissed,

            // ── Vim-style quit ───────────────────────────────────────────────
            KeyCode::Char('q') if key.modifiers.is_empty() => ExplorerOutcome::Dismissed,

            // ── Move up ──────────────────────────────────────────────────────
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                ExplorerOutcome::Pending
            }

            // ── Move down ────────────────────────────────────────────────────
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
                ExplorerOutcome::Pending
            }

            // ── Page up ──────────────────────────────────────────────────────
            KeyCode::PageUp => {
                for _ in 0..10 {
                    self.move_up();
                }
                ExplorerOutcome::Pending
            }

            // ── Page down ────────────────────────────────────────────────────
            KeyCode::PageDown => {
                for _ in 0..10 {
                    self.move_down();
                }
                ExplorerOutcome::Pending
            }

            // ── Jump to top ──────────────────────────────────────────────────
            KeyCode::Home | KeyCode::Char('g') => {
                self.cursor = 0;
                self.scroll_offset = 0;
                ExplorerOutcome::Pending
            }

            // ── Jump to bottom ───────────────────────────────────────────────
            KeyCode::End | KeyCode::Char('G') => {
                if !self.entries.is_empty() {
                    self.cursor = self.entries.len() - 1;
                }
                ExplorerOutcome::Pending
            }

            // ── Ascend (go to parent) ─────────────────────────────────────────
            KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') => {
                self.ascend();
                ExplorerOutcome::Pending
            }

            // ── Confirm / descend ─────────────────────────────────────────────
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => self.confirm(),

            // ── Toggle hidden files ───────────────────────────────────────────
            KeyCode::Char('.') => {
                self.show_hidden = !self.show_hidden;
                let was = self.cursor;
                self.reload();
                self.cursor = was.min(self.entries.len().saturating_sub(1));
                ExplorerOutcome::Pending
            }

            _ => ExplorerOutcome::Unhandled,
        }
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    /// The currently highlighted [`FsEntry`], or `None` if the list is empty.
    pub fn current_entry(&self) -> Option<&FsEntry> {
        self.entries.get(self.cursor)
    }

    // ── Internal navigation helpers ───────────────────────────────────────────

    fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    fn move_down(&mut self) {
        if !self.entries.is_empty() && self.cursor < self.entries.len() - 1 {
            self.cursor += 1;
        }
    }

    fn ascend(&mut self) {
        if let Some(parent) = self.current_dir.parent().map(|p| p.to_path_buf()) {
            let prev = self.current_dir.clone();
            self.current_dir = parent;
            self.cursor = 0;
            self.scroll_offset = 0;
            self.reload();
            // Try to land the cursor on the directory we just came from.
            if let Some(idx) = self.entries.iter().position(|e| e.path == prev) {
                self.cursor = idx;
            }
        } else {
            self.status = "Already at the filesystem root.".to_string();
        }
    }

    fn confirm(&mut self) -> ExplorerOutcome {
        let Some(entry) = self.entries.get(self.cursor) else {
            return ExplorerOutcome::Pending;
        };

        if entry.is_dir {
            let path = entry.path.clone();
            self.navigate_to(path);
            ExplorerOutcome::Pending
        } else if self.is_selectable(entry) {
            ExplorerOutcome::Selected(entry.path.clone())
        } else {
            self.status = format!("Not a supported file type. Allowed: {}", self.filter_hint());
            ExplorerOutcome::Pending
        }
    }

    fn is_selectable(&self, entry: &FsEntry) -> bool {
        if entry.is_dir {
            return false;
        }
        if self.extension_filter.is_empty() {
            return true;
        }
        self.extension_filter
            .iter()
            .any(|ext| ext.eq_ignore_ascii_case(&entry.extension))
    }

    fn filter_hint(&self) -> String {
        if self.extension_filter.is_empty() {
            "*".to_string()
        } else {
            self.extension_filter
                .iter()
                .map(|e| format!(".{e}"))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }

    // ── Directory loading ─────────────────────────────────────────────────────

    /// Re-read the current directory from the filesystem.
    pub fn reload(&mut self) {
        self.status.clear();
        self.entries = load_entries(&self.current_dir, self.show_hidden, &self.extension_filter);
    }
}

// ── Filesystem helpers ────────────────────────────────────────────────────────

fn load_entries(dir: &Path, show_hidden: bool, ext_filter: &[String]) -> Vec<FsEntry> {
    let read = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut dirs: Vec<FsEntry> = Vec::new();
    let mut files: Vec<FsEntry> = Vec::new();

    for entry in read.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if !show_hidden && name.starts_with('.') {
            continue;
        }

        let is_dir = path.is_dir();
        let extension = if is_dir {
            String::new()
        } else {
            path.extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default()
        };

        // Exclude files that don't match the filter (dirs always shown).
        if !is_dir && !ext_filter.is_empty() {
            let matches = ext_filter
                .iter()
                .any(|f| f.eq_ignore_ascii_case(&extension));
            if !matches {
                continue;
            }
        }

        let size = if is_dir {
            None
        } else {
            entry.metadata().ok().map(|m| m.len())
        };

        let fs_entry = FsEntry {
            name,
            path,
            is_dir,
            size,
            extension,
        };

        if is_dir {
            dirs.push(fs_entry);
        } else {
            files.push(fs_entry);
        }
    }

    // Sort each group alphabetically (case-insensitive).
    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Dirs first, then matching files.
    dirs.extend(files);
    dirs
}

// ── Icons ─────────────────────────────────────────────────────────────────────

fn entry_icon(entry: &FsEntry) -> &'static str {
    if entry.is_dir {
        return "📁";
    }
    match entry.extension.as_str() {
        "iso" => "💿",
        "img" => "🖼 ",
        "zip" | "gz" | "xz" | "zst" | "bz2" | "tar" => "📦",
        "txt" | "md" | "rst" => "📄",
        "sh" | "bash" | "zsh" | "fish" => "📜",
        "toml" | "yaml" | "yml" | "json" => "⚙ ",
        _ => "📄",
    }
}

fn fmt_size(bytes: u64) -> String {
    const KB: u64 = 1_024;
    const MB: u64 = 1_024 * KB;
    const GB: u64 = 1_024 * MB;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

// ── Rendering ─────────────────────────────────────────────────────────────────

/// Render the file explorer into `area`.
///
/// This is the single rendering entry-point. Call it from your application's
/// `render` function, passing a mutable reference to the explorer state and
/// the current Ratatui [`Frame`].
///
/// The widget takes full ownership of `area` and renders:
/// * A header block showing the current directory path.
/// * A scrollable, highlighted list of directory entries.
/// * A footer bar with key hints and a status message.
pub fn render(explorer: &mut FileExplorer, frame: &mut Frame, area: Rect) {
    // ── Layout: [header(3)] [list(fill)] [footer(3)] ─────────────────────────
    let [header_area, list_area, footer_area] = {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(area);
        [chunks[0], chunks[1], chunks[2]]
    };

    render_header(explorer, frame, header_area);
    render_list(explorer, frame, list_area);
    render_footer(explorer, frame, footer_area);
}

// ── Header ────────────────────────────────────────────────────────────────────

fn render_header(explorer: &FileExplorer, frame: &mut Frame, area: Rect) {
    let path_str = explorer.current_dir.to_string_lossy();

    // Truncate from the left if the path is wider than the available area.
    let inner_width = area.width.saturating_sub(4) as usize; // account for borders + padding
    let display_path = if path_str.len() > inner_width && inner_width > 3 {
        let skip = path_str.len() - inner_width + 1;
        format!("…{}", &path_str[skip..])
    } else {
        path_str.to_string()
    };

    let header = Paragraph::new(Span::styled(
        display_path,
        Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
    ))
    .block(
        Block::default()
            .title(Span::styled(
                " 📁  File Explorer ",
                Style::default().fg(C_BRAND).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(C_ACCENT))
            .padding(Padding::horizontal(1)),
    )
    .alignment(Alignment::Left);

    frame.render_widget(header, area);
}

// ── Entry list ────────────────────────────────────────────────────────────────

fn render_list(explorer: &mut FileExplorer, frame: &mut Frame, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize; // subtract border rows

    // Keep scroll_offset in sync so the cursor is always visible.
    if explorer.cursor < explorer.scroll_offset {
        explorer.scroll_offset = explorer.cursor;
    } else if explorer.cursor >= explorer.scroll_offset + visible_height {
        explorer.scroll_offset = explorer.cursor - visible_height + 1;
    }

    let items: Vec<ListItem> = explorer
        .entries
        .iter()
        .skip(explorer.scroll_offset)
        .take(visible_height)
        .enumerate()
        .map(|(visible_idx, entry)| {
            let abs_idx = visible_idx + explorer.scroll_offset;
            let is_selected = abs_idx == explorer.cursor;
            let is_selectable = explorer.is_selectable(entry);

            // ── Icon + name ───────────────────────────────────────────────────
            let icon = entry_icon(entry);

            // Choose colours based on entry type and selectability.
            let name_style = if entry.is_dir {
                Style::default().fg(C_DIR).add_modifier(Modifier::BOLD)
            } else if is_selectable {
                Style::default().fg(C_MATCH).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(C_DIM)
            };

            // Size column (right-aligned): only for files.
            let size_str = match entry.size {
                Some(b) => fmt_size(b),
                None => String::new(),
            };

            // Build the line with a leading space for padding.
            let mut spans = vec![
                Span::styled(" ", Style::default()),
                Span::styled(
                    format!("{icon} "),
                    Style::default().fg(if entry.is_dir { C_DIR } else { C_FG }),
                ),
                Span::styled(entry.name.clone(), name_style),
            ];

            if !size_str.is_empty() {
                spans.push(Span::styled(
                    format!("  {size_str}"),
                    Style::default().fg(C_DIM),
                ));
            }

            if entry.is_dir {
                spans.push(Span::styled("/", Style::default().fg(C_DIR)));
            }

            let line = Line::from(spans);

            // Highlight the selected row.
            if is_selected {
                ListItem::new(line)
                    .style(Style::default().bg(C_SEL_BG).add_modifier(Modifier::BOLD))
            } else {
                ListItem::new(line)
            }
        })
        .collect();

    // Scroll indicator in the title (e.g. "3/42").
    let count = explorer.entries.len();
    let pos = if count == 0 {
        "empty".to_string()
    } else {
        format!("{}/{count}", explorer.cursor + 1)
    };

    let block = Block::default()
        .title(Span::styled(
            format!(" Files {pos} "),
            Style::default().fg(C_DIM),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(C_ACCENT));

    let mut list_state = ListState::default();
    // ListState offset tracks the scroll; we manage it manually above but
    // still need to tell Ratatui which item is "selected" for highlighting.
    // We pass the visible index so the internal highlight doesn't double-draw.
    if !explorer.entries.is_empty() {
        list_state.select(Some(explorer.cursor.saturating_sub(explorer.scroll_offset)));
    }

    let list = List::new(items).block(block);
    frame.render_stateful_widget(list, area, &mut list_state);
}

// ── Footer ────────────────────────────────────────────────────────────────────

fn render_footer(explorer: &FileExplorer, frame: &mut Frame, area: Rect) {
    // Left: key hints   Right: status message
    let hints = " ↑/k Up  ↓/j Down  Enter Confirm  ← Ascend  . Hidden  Esc Dismiss";

    let status = if explorer.status.is_empty() {
        let filter = if explorer.extension_filter.is_empty() {
            "all files".to_string()
        } else {
            explorer
                .extension_filter
                .iter()
                .map(|e| format!(".{e}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let hidden_hint = if explorer.show_hidden {
            " [+hidden]"
        } else {
            ""
        };
        format!("Filter: {filter}{hidden_hint}")
    } else {
        explorer.status.clone()
    };

    let [left_area, right_area] = {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(30)])
            .split(area);
        [chunks[0], chunks[1]]
    };

    let hints_para = Paragraph::new(Span::styled(hints, Style::default().fg(C_DIM))).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(C_DIM)),
    );

    let status_para = Paragraph::new(Span::styled(status, Style::default().fg(C_SUCCESS)))
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(C_DIM)),
        );

    frame.render_widget(hints_para, left_area);
    frame.render_widget(status_para, right_area);
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;
    use std::fs;
    use tempfile::TempDir;

    fn temp_dir_with_files() -> TempDir {
        let dir = tempfile::tempdir().expect("temp dir");
        fs::write(dir.path().join("ubuntu.iso"), b"fake iso content").unwrap();
        fs::write(dir.path().join("debian.img"), b"fake img content").unwrap();
        fs::write(dir.path().join("readme.txt"), b"some text").unwrap();
        fs::create_dir(dir.path().join("subdir")).unwrap();
        dir
    }

    #[test]
    fn new_loads_entries() {
        let tmp = temp_dir_with_files();
        let explorer =
            FileExplorer::new(tmp.path().to_path_buf(), vec!["iso".into(), "img".into()]);
        // Dirs come first, then filtered files.
        assert!(explorer
            .entries
            .iter()
            .any(|e| e.name == "subdir" && e.is_dir));
        assert!(explorer.entries.iter().any(|e| e.name == "ubuntu.iso"));
        assert!(explorer.entries.iter().any(|e| e.name == "debian.img"));
        // .txt should be excluded by the filter.
        assert!(!explorer.entries.iter().any(|e| e.name == "readme.txt"));
    }

    #[test]
    fn no_filter_shows_all_files() {
        let tmp = temp_dir_with_files();
        let explorer = FileExplorer::new(tmp.path().to_path_buf(), vec![]);
        assert!(explorer.entries.iter().any(|e| e.name == "readme.txt"));
    }

    #[test]
    fn dirs_listed_before_files() {
        let tmp = temp_dir_with_files();
        let explorer = FileExplorer::new(tmp.path().to_path_buf(), vec![]);
        let first_file_idx = explorer
            .entries
            .iter()
            .position(|e| !e.is_dir)
            .unwrap_or(usize::MAX);
        let last_dir_idx = explorer.entries.iter().rposition(|e| e.is_dir).unwrap_or(0);
        assert!(
            last_dir_idx < first_file_idx,
            "dirs should appear before files"
        );
    }

    #[test]
    fn move_down_increments_cursor() {
        let tmp = temp_dir_with_files();
        let mut explorer = FileExplorer::new(tmp.path().to_path_buf(), vec![]);
        assert_eq!(explorer.cursor, 0);
        explorer.move_down();
        assert_eq!(explorer.cursor, 1);
    }

    #[test]
    fn move_up_clamps_at_zero() {
        let tmp = temp_dir_with_files();
        let mut explorer = FileExplorer::new(tmp.path().to_path_buf(), vec![]);
        explorer.move_up();
        assert_eq!(explorer.cursor, 0);
    }

    #[test]
    fn move_down_clamps_at_last() {
        let tmp = temp_dir_with_files();
        let mut explorer = FileExplorer::new(tmp.path().to_path_buf(), vec![]);
        let last = explorer.entries.len() - 1;
        explorer.cursor = last;
        explorer.move_down();
        assert_eq!(explorer.cursor, last);
    }

    #[test]
    fn handle_key_down_moves_cursor() {
        let tmp = temp_dir_with_files();
        let mut explorer = FileExplorer::new(tmp.path().to_path_buf(), vec![]);
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let outcome = explorer.handle_key(key);
        assert_eq!(outcome, ExplorerOutcome::Pending);
        assert_eq!(explorer.cursor, 1);
    }

    #[test]
    fn handle_key_esc_dismisses() {
        let tmp = temp_dir_with_files();
        let mut explorer = FileExplorer::new(tmp.path().to_path_buf(), vec![]);
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(explorer.handle_key(key), ExplorerOutcome::Dismissed);
    }

    #[test]
    fn handle_key_enter_on_dir_descends() {
        let tmp = temp_dir_with_files();
        let mut explorer = FileExplorer::new(tmp.path().to_path_buf(), vec![]);
        // subdir is the first entry (dirs come first).
        let subdir_idx = explorer
            .entries
            .iter()
            .position(|e| e.is_dir)
            .expect("no dirs");
        explorer.cursor = subdir_idx;
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let outcome = explorer.handle_key(key);
        assert_eq!(outcome, ExplorerOutcome::Pending);
        // Current dir should now be the subdir.
        assert!(explorer.current_dir.ends_with("subdir"));
    }

    #[test]
    fn handle_key_enter_on_valid_file_selects() {
        let tmp = temp_dir_with_files();
        let mut explorer =
            FileExplorer::new(tmp.path().to_path_buf(), vec!["iso".into(), "img".into()]);
        // Find the ubuntu.iso entry.
        let iso_idx = explorer
            .entries
            .iter()
            .position(|e| e.name == "ubuntu.iso")
            .expect("ubuntu.iso not found");
        explorer.cursor = iso_idx;
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let outcome = explorer.handle_key(key);
        assert!(matches!(outcome, ExplorerOutcome::Selected(_)));
        if let ExplorerOutcome::Selected(p) = outcome {
            assert!(p.ends_with("ubuntu.iso"));
        }
    }

    #[test]
    fn handle_key_backspace_ascends() {
        let tmp = temp_dir_with_files();
        let subdir = tmp.path().join("subdir");
        let mut explorer = FileExplorer::new(subdir, vec![]);
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        explorer.handle_key(key);
        assert_eq!(explorer.current_dir, tmp.path());
    }

    #[test]
    fn toggle_hidden_changes_visibility() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join(".hidden"), b"x").unwrap();
        fs::write(tmp.path().join("visible.txt"), b"y").unwrap();

        let mut explorer = FileExplorer::new(tmp.path().to_path_buf(), vec![]);
        assert!(!explorer.entries.iter().any(|e| e.name == ".hidden"));

        let key = KeyEvent::new(KeyCode::Char('.'), KeyModifiers::NONE);
        explorer.handle_key(key);
        assert!(explorer.entries.iter().any(|e| e.name == ".hidden"));
    }

    #[test]
    fn fmt_size_formats_bytes() {
        assert_eq!(fmt_size(512), "512 B");
        assert_eq!(fmt_size(1_536), "1.5 KB");
        assert_eq!(fmt_size(2_097_152), "2.0 MB");
        assert_eq!(fmt_size(1_073_741_824), "1.0 GB");
    }

    #[test]
    fn is_selectable_respects_filter() {
        let tmp = temp_dir_with_files();
        // Use a filter of ["iso"] only — img files should not be selectable.
        let explorer = FileExplorer::new(tmp.path().to_path_buf(), vec!["iso".into()]);

        // Build synthetic entries to test is_selectable without depending on
        // what load_entries includes (debian.img is filtered out of the list).
        let iso_entry = FsEntry {
            name: "ubuntu.iso".into(),
            path: tmp.path().join("ubuntu.iso"),
            is_dir: false,
            size: Some(16),
            extension: "iso".into(),
        };
        let img_entry = FsEntry {
            name: "debian.img".into(),
            path: tmp.path().join("debian.img"),
            is_dir: false,
            size: Some(16),
            extension: "img".into(),
        };
        let dir_entry = FsEntry {
            name: "subdir".into(),
            path: tmp.path().join("subdir"),
            is_dir: true,
            size: None,
            extension: String::new(),
        };

        assert!(
            explorer.is_selectable(&iso_entry),
            "iso should be selectable"
        );
        assert!(
            !explorer.is_selectable(&img_entry),
            "img should not be selectable with iso-only filter"
        );
        assert!(
            !explorer.is_selectable(&dir_entry),
            "dirs are never selectable"
        );
    }

    #[test]
    fn navigate_to_resets_cursor_and_scroll() {
        let tmp = temp_dir_with_files();
        let mut explorer = FileExplorer::new(tmp.path().to_path_buf(), vec![]);
        explorer.cursor = 2;
        explorer.scroll_offset = 1;
        explorer.navigate_to(tmp.path().to_path_buf());
        assert_eq!(explorer.cursor, 0);
        assert_eq!(explorer.scroll_offset, 0);
    }

    #[test]
    fn current_entry_returns_highlighted() {
        let tmp = temp_dir_with_files();
        let mut explorer = FileExplorer::new(tmp.path().to_path_buf(), vec![]);
        explorer.cursor = 0;
        let entry = explorer.current_entry().expect("should have entry");
        assert_eq!(entry, explorer.entries.first().unwrap());
    }

    #[test]
    fn unrecognised_key_returns_unhandled() {
        let tmp = temp_dir_with_files();
        let mut explorer = FileExplorer::new(tmp.path().to_path_buf(), vec![]);
        let key = KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE);
        assert_eq!(explorer.handle_key(key), ExplorerOutcome::Unhandled);
    }
}
