# VHS tape for FlashKraft - Theme Storage & Persistence Demo
# This demonstrates how FlashKraft persists theme selection using sled database

Output examples/vhs/demo-storage.gif
Set FontSize 14
Set Width 1400
Set Height 900
Set Theme "Dracula"
Set PlaybackSpeed 0.6

# Title Screen
Type "╔════════════════════════════════════════════════════════════╗"
Enter
Type "║         FlashKraft - Storage & Persistence                ║"
Enter
Type "║         Theme Selection with Sled Database                ║"
Enter
Type "╚════════════════════════════════════════════════════════════╝"
Enter
Sleep 3s

Enter
Type "Press any key to continue..."
Sleep 2s
Enter
Enter

# Section 1: Introduction
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "1. Theme Persistence System"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "FlashKraft remembers your theme preference across sessions"
Enter
Type "using an embedded key-value database called Sled."
Enter
Sleep 2s

Enter
Type "Why Sled?"
Enter
Type "  ✓ Embedded - No external database needed"
Enter
Type "  ✓ Fast - Lock-free, concurrent access"
Enter
Type "  ✓ Reliable - ACID transactions"
Enter
Type "  ✓ Simple - Key-value store interface"
Enter
Type "  ✓ Lightweight - Small footprint"
Enter
Sleep 3s

Enter
Enter
Screenshot examples/vhs/screenshots/storage-01-intro.png
Sleep 1s

# Section 2: Storage Module
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "2. Storage Module Implementation"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "// src/core/storage.rs"
Enter
Sleep 1s

Enter
Type "use sled::{Db, Result as SledResult};"
Enter
Type "use iced::Theme;"
Enter
Enter
Type "pub struct Storage {"
Enter
Type "    db: Db,"
Enter
Type "}"
Enter
Sleep 2s

Enter
Enter
Type "impl Storage {"
Enter
Type "    pub fn new() -> SledResult<Self> {"
Enter
Type "        let db = sled::open(Self::db_path())?;"
Enter
Type "        Ok(Self { db })"
Enter
Type "    }"
Enter
Sleep 2s

Enter
Enter
Type "    pub fn save_theme(&self, theme: &Theme) {"
Enter
Type "        let theme_str = Self::theme_to_string(theme);"
Enter
Type "        self.db.insert(b\"theme\", theme_str.as_bytes());"
Enter
Type "    }"
Enter
Sleep 2s

Enter
Enter
Type "    pub fn load_theme(&self) -> Option<Theme> {"
Enter
Type "        self.db.get(b\"theme\").ok()?"
Enter
Type "            .map(|bytes| Self::string_to_theme(&bytes))"
Enter
Type "    }"
Enter
Type "}"
Enter
Sleep 3s

Enter
Enter
Screenshot examples/vhs/screenshots/storage-02-module.png
Sleep 1s

# Section 3: Database Location
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "3. Database Location"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "FlashKraft stores its database in platform-specific locations:"
Enter
Sleep 1s

Enter
Type "🐧 Linux:"
Enter
Type "   ~/.local/share/flashkraft/db/"
Enter
Sleep 1.5s

Enter
Type "🍎 macOS:"
Enter
Type "   ~/Library/Application Support/flashkraft/db/"
Enter
Sleep 1.5s

Enter
Type "🪟 Windows:"
Enter
Type "   %APPDATA%\\flashkraft\\db\\"
Enter
Sleep 2s

Enter
Enter
Type "Let's check the actual location:"
Enter
Sleep 1s

Enter
Type "fn db_path() -> PathBuf {"
Enter
Type "    let base = dirs::data_local_dir()"
Enter
Type "        .unwrap_or_else(|| PathBuf::from(\".\"));"
Enter
Type "    base.join(\"flashkraft\").join(\"db\")"
Enter
Type "}"
Enter
Sleep 3s

Enter
Enter
Screenshot examples/vhs/screenshots/storage-03-location.png
Sleep 1s

# Section 4: Theme Serialization
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "4. Theme Serialization"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "Converting themes to/from strings for storage:"
Enter
Sleep 1s

Enter
Type "fn theme_to_string(theme: &Theme) -> String {"
Enter
Type "    match theme {"
Enter
Type "        Theme::Dark => \"dark\","
Enter
Type "        Theme::Light => \"light\","
Enter
Type "        Theme::Dracula => \"dracula\","
Enter
Type "        Theme::Nord => \"nord\","
Enter
Type "        Theme::TokyoNight => \"tokyonight\","
Enter
Type "        Theme::CatppuccinMocha => \"catppuccin_mocha\","
Enter
Type "        Theme::GruvboxDark => \"gruvbox_dark\","
Enter
Type "        Theme::KanagawaWave => \"kanagawa_wave\","
Enter
Type "        _ => \"dark\","
Enter
Type "    }.to_string()"
Enter
Type "}"
Enter
Sleep 3s

Enter
Enter
Type "fn string_to_theme(s: &[u8]) -> Theme {"
Enter
Type "    match std::str::from_utf8(s).unwrap_or(\"dark\") {"
Enter
Type "        \"light\" => Theme::Light,"
Enter
Type "        \"dracula\" => Theme::Dracula,"
Enter
Type "        \"nord\" => Theme::Nord,"
Enter
Type "        // ... other themes"
Enter
Type "        _ => Theme::Dark,"
Enter
Type "    }"
Enter
Type "}"
Enter
Sleep 3s

Enter
Enter
Screenshot examples/vhs/screenshots/storage-04-serialization.png
Sleep 1s

# Section 5: Integration with State
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "5. Integration with Application State"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "// src/core/state.rs"
Enter
Sleep 1s

Enter
Type "impl FlashKraft {"
Enter
Type "    pub fn new() -> (Self, Command<Message>) {"
Enter
Type "        let storage = Storage::new().ok();"
Enter
Enter
Type "        // Load saved theme or use default"
Enter
Type "        let theme = storage"
Enter
Type "            .as_ref()"
Enter
Type "            .and_then(|s| s.load_theme())"
Enter
Type "            .unwrap_or(Theme::Dark);"
Enter
Sleep 2s

Enter
Enter
Type "        let state = Self {"
Enter
Type "            theme,"
Enter
Type "            storage,"
Enter
Type "            // ... other fields"
Enter
Type "        };"
Enter
Enter
Type "        (state, Command::none())"
Enter
Type "    }"
Enter
Type "}"
Enter
Sleep 3s

Enter
Enter
Screenshot examples/vhs/screenshots/storage-05-integration.png
Sleep 1s

# Section 6: Saving Theme Changes
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "6. Saving Theme Changes"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "When user changes theme, we persist it immediately:"
Enter
Sleep 1s

Enter
Type "// src/core/update.rs"
Enter
Enter
Type "fn update(&mut self, message: Message) -> Command<Message> {"
Enter
Type "    match message {"
Enter
Type "        Message::ThemeChanged(new_theme) => {"
Enter
Sleep 1s

Enter
Type "            // Update state"
Enter
Type "            self.theme = new_theme.clone();"
Enter
Sleep 1s

Enter
Type "            // Persist to database"
Enter
Type "            if let Some(storage) = &self.storage {"
Enter
Type "                storage.save_theme(&new_theme);"
Enter
Type "            }"
Enter
Sleep 1s

Enter
Type "            Command::none()"
Enter
Type "        }"
Enter
Type "        // ... other messages"
Enter
Type "    }"
Enter
Type "}"
Enter
Sleep 3s

Enter
Enter
Screenshot examples/vhs/screenshots/storage-06-saving.png
Sleep 1s

# Section 7: Workflow Example
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "7. Complete Workflow Example"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "Session 1: User selects Dracula theme"
Enter
Type "───────────────────────────────────────"
Enter
Sleep 1s

Enter
Type "1. User clicks theme dropdown"
Enter
Type "   UI → Message::ThemeChanged(Theme::Dracula)"
Enter
Sleep 1.5s

Enter
Type "2. Update function processes message"
Enter
Type "   self.theme = Theme::Dracula"
Enter
Sleep 1.5s

Enter
Type "3. Save to database"
Enter
Type "   storage.save_theme(&Theme::Dracula)"
Enter
Sleep 1.5s

Enter
Type "4. Sled writes to disk"
Enter
Type "   db.insert(\"theme\", \"dracula\")"
Enter
Sleep 1.5s

Enter
Type "5. UI re-renders with new theme"
Enter
Type "   view() → Dracula colors applied"
Enter
Sleep 3s

Enter
Enter
Type "Session 2: User restarts application"
Enter
Type "───────────────────────────────────────"
Enter
Sleep 1s

Enter
Type "1. Application initializes"
Enter
Type "   FlashKraft::new() called"
Enter
Sleep 1.5s

Enter
Type "2. Storage loads from database"
Enter
Type "   storage.load_theme()"
Enter
Sleep 1.5s

Enter
Type "3. Database returns saved value"
Enter
Type "   db.get(\"theme\") → \"dracula\""
Enter
Sleep 1.5s

Enter
Type "4. String converted to Theme"
Enter
Type "   string_to_theme(\"dracula\") → Theme::Dracula"
Enter
Sleep 1.5s

Enter
Type "5. Application starts with Dracula theme!"
Enter
Type "   User's preference is preserved ✓"
Enter
Sleep 3s

Enter
Enter
Screenshot examples/vhs/screenshots/storage-07-workflow.png
Sleep 1s

# Section 8: Error Handling
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "8. Error Handling & Fallbacks"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "FlashKraft handles storage errors gracefully:"
Enter
Sleep 1s

Enter
Type "🔧 Database Creation Failure"
Enter
Type "   → Falls back to in-memory operation"
Enter
Type "   → Application still works, just no persistence"
Enter
Sleep 2s

Enter
Type "🔧 Read Failure"
Enter
Type "   → Returns None, uses default theme"
Enter
Type "   → Logs warning but doesn't crash"
Enter
Sleep 2s

Enter
Type "🔧 Write Failure"
Enter
Type "   → Silently fails, theme change still applies"
Enter
Type "   → User sees immediate feedback in UI"
Enter
Sleep 2s

Enter
Type "🔧 Corrupt Data"
Enter
Type "   → Invalid theme string → defaults to Dark"
Enter
Type "   → UTF-8 decode error → defaults to Dark"
Enter
Sleep 3s

Enter
Enter
Type "Example error handling:"
Enter
Enter
Type "let storage = Storage::new().ok();  // Returns Option<Storage>"
Enter
Type "// If None, storage features disabled but app works"
Enter
Sleep 3s

Enter
Enter
Screenshot examples/vhs/screenshots/storage-08-errors.png
Sleep 1s

# Section 9: Benefits
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "9. Benefits of Sled Storage"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "✓ NO EXTERNAL DEPENDENCIES"
Enter
Type "  • No PostgreSQL, MySQL, SQLite to install"
Enter
Type "  • Embedded directly in the application"
Enter
Type "  • Single binary distribution"
Enter
Sleep 2s

Enter
Type "✓ INSTANT STARTUP"
Enter
Type "  • Database opens in microseconds"
Enter
Type "  • No connection overhead"
Enter
Type "  • No network latency"
Enter
Sleep 2s

Enter
Type "✓ ZERO CONFIGURATION"
Enter
Type "  • Works out of the box"
Enter
Type "  • Auto-creates database on first run"
Enter
Type "  • Platform-aware path selection"
Enter
Sleep 2s

Enter
Type "✓ CONCURRENT ACCESS"
Enter
Type "  • Lock-free data structures"
Enter
Type "  • Multiple readers, single writer"
Enter
Type "  • Perfect for GUI apps"
Enter
Sleep 2s

Enter
Type "✓ CRASH RESISTANCE"
Enter
Type "  • ACID transactions"
Enter
Type "  • Automatic recovery"
Enter
Type "  • Data integrity guaranteed"
Enter
Sleep 2s

Enter
Type "✓ SMALL FOOTPRINT"
Enter
Type "  • Database file: ~100 KB"
Enter
Type "  • Memory usage: minimal"
Enter
Type "  • Binary size: only ~500 KB added"
Enter
Sleep 3s

Enter
Enter
Screenshot examples/vhs/screenshots/storage-09-benefits.png
Sleep 1s

# Section 10: Future Enhancements
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "10. Future Storage Enhancements"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "🚀 Planned Features:"
Enter
Sleep 1s

Enter
Type "📝 More Settings to Persist"
Enter
Type "   • Last selected image path"
Enter
Type "   • Last selected drive"
Enter
Type "   • Window size and position"
Enter
Type "   • User preferences"
Enter
Sleep 2s

Enter
Type "📊 Usage Statistics"
Enter
Type "   • Number of successful flashes"
Enter
Type "   • Average flash time"
Enter
Type "   • Most used drives"
Enter
Type "   • Error history"
Enter
Sleep 2s

Enter
Type "🕐 Recent History"
Enter
Type "   • Recent images flashed"
Enter
Type "   • Quick re-flash feature"
Enter
Type "   • Favorite images list"
Enter
Sleep 2s

Enter
Type "🎨 Custom Themes"
Enter
Type "   • User-created themes"
Enter
Type "   • Import/export themes"
Enter
Type "   • Theme sharing"
Enter
Sleep 2s

Enter
Type "💾 Export/Import Settings"
Enter
Type "   • Backup user preferences"
Enter
Type "   • Sync across devices"
Enter
Type "   • Settings migration"
Enter
Sleep 3s

Enter
Enter
Screenshot examples/vhs/screenshots/storage-10-future.png
Sleep 1s

# Summary
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "Summary - Storage & Persistence"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "FlashKraft's storage system provides:"
Enter
Sleep 1s

Enter
Type "✓ Persistent theme selection across sessions"
Enter
Type "✓ Embedded database (no external dependencies)"
Enter
Type "✓ Fast, reliable, crash-resistant storage"
Enter
Type "✓ Graceful error handling with fallbacks"
Enter
Type "✓ Platform-aware file locations"
Enter
Type "✓ Extensible for future features"
Enter
Sleep 3s

Enter
Enter
Type "Technology Stack:"
Enter
Type "  • Sled - Embedded key-value database"
Enter
Type "  • dirs - Platform-specific paths"
Enter
Type "  • Rust - Memory safety and reliability"
Enter
Sleep 3s

Enter
Enter
Screenshot examples/vhs/screenshots/storage-11-summary.png
Sleep 2s

Enter
Enter
Type "╔════════════════════════════════════════════════════════════╗"
Enter
Type "║           Storage System Deep Dive Complete!              ║"
Enter
Type "║                                                           ║"
Enter
Type "║   See: src/core/storage.rs for implementation            ║"
Enter
Type "║   Try: Change themes and restart the app!                ║"
Enter
Type "╚════════════════════════════════════════════════════════════╝"
Enter
Sleep 4s
