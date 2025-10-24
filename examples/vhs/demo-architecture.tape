# VHS tape for FlashKraft - The Elm Architecture Explanation
# This tape provides a visual explanation of The Elm Architecture

Output examples/vhs/demo-architecture.gif
Set FontSize 16
Set Width 1400
Set Height 900
Set Theme "Nord"
Set PlaybackSpeed 0.8

# Title
Type "# The Elm Architecture in FlashKraft"
Enter
Sleep 2s
Clear

# Introduction
Type "# What is The Elm Architecture?"
Enter
Sleep 1s
Enter
Type "A pattern for building interactive applications with:"
Enter
Type "  1. Model  - Application state"
Enter
Type "  2. Message - Events that can occur"
Enter
Type "  3. Update - State transition logic"
Enter
Type "  4. View   - UI rendering"
Enter
Sleep 4s
Clear

# Visual diagram
Type "# Data Flow"
Enter
Sleep 1s
Enter
Type "┌─────────────────────────────────────────────────────────┐"
Enter
Type "│                                                         │"
Enter
Type "│  User Input → Message → Update → State → View → UI     │"
Enter
Type "│                           ↓                             │"
Enter
Type "│                        Command                          │"
Enter
Type "│                           ↓                             │"
Enter
Type "│          Async Result → Message → Update...             │"
Enter
Type "│                                                         │"
Enter
Type "└─────────────────────────────────────────────────────────┘"
Enter
Sleep 5s
Clear

# Model explanation
Type "# 1. MODEL - Application State"
Enter
Sleep 1s
Enter
Type "cat << 'EOF'"
Enter
Type "struct FlashKraft {"
Enter
Type "    selected_image: Option<ImageInfo>,"
Enter
Type "    selected_target: Option<DriveInfo>,"
Enter
Type "    available_drives: Vec<DriveInfo>,"
Enter
Type "    flash_progress: Option<f32>,"
Enter
Type "    error_message: Option<String>,"
Enter
Type "}"
Enter
Type "EOF"
Enter
Sleep 4s
Screenshot examples/vhs/screenshots/arch-01-model.png
Clear

# Message explanation
Type "# 2. MESSAGE - Events"
Enter
Sleep 1s
Enter
Type "cat << 'EOF'"
Enter
Type "enum Message {"
Enter
Type "    // User interactions"
Enter
Type "    SelectImageClicked,"
Enter
Type "    RefreshDrivesClicked,"
Enter
Type "    FlashClicked,"
Enter
Type "    "
Enter
Type "    // Async results"
Enter
Type "    ImageSelected(Option<PathBuf>),"
Enter
Type "    DrivesRefreshed(Vec<DriveInfo>),"
Enter
Type "    FlashCompleted(Result<(), String>),"
Enter
Type "}"
Enter
Type "EOF"
Enter
Sleep 4s
Screenshot examples/vhs/screenshots/arch-02-message.png
Clear

# Update explanation
Type "# 3. UPDATE - State Transitions"
Enter
Sleep 1s
Enter
Type "cat << 'EOF'"
Enter
Type "fn update(&mut self, message: Message) -> Task<Message> {"
Enter
Type "    match message {"
Enter
Type "        Message::SelectImageClicked => {"
Enter
Type "            // Trigger async file selection"
Enter
Type "            Task::perform("
Enter
Type "                select_file(),"
Enter
Type "                Message::ImageSelected"
Enter
Type "            )"
Enter
Type "        }"
Enter
Type "        Message::ImageSelected(path) => {"
Enter
Type "            self.selected_image = path;"
Enter
Type "            Task::none()"
Enter
Type "        }"
Enter
Type "        // ... handle other messages"
Enter
Type "    }"
Enter
Type "}"
Enter
Type "EOF"
Enter
Sleep 4s
Screenshot examples/vhs/screenshots/arch-03-update.png
Clear

# View explanation
Type "# 4. VIEW - UI Rendering"
Enter
Sleep 1s
Enter
Type "cat << 'EOF'"
Enter
Type "fn view(&self) -> Element<Message> {"
Enter
Type "    if let Some(progress) = self.flash_progress {"
Enter
Type "        // Show progress bar"
Enter
Type "        view_flashing(progress)"
Enter
Type "    } else if let Some(error) = &self.error_message {"
Enter
Type "        // Show error"
Enter
Type "        view_error(error)"
Enter
Type "    } else {"
Enter
Type "        // Show main UI"
Enter
Type "        view_main(self)"
Enter
Type "    }"
Enter
Type "}"
Enter
Type "EOF"
Enter
Sleep 4s
Screenshot examples/vhs/screenshots/arch-04-view.png
Clear

# Benefits
Type "# Benefits of The Elm Architecture"
Enter
Sleep 1s
Enter
Type "✓ Predictable  - State flows in one direction"
Enter
Sleep 1s
Type "✓ Testable     - Pure functions, easy to test"
Enter
Sleep 1s
Type "✓ Debuggable   - All state changes are explicit"
Enter
Sleep 1s
Type "✓ Maintainable - Clear separation of concerns"
Enter
Sleep 1s
Type "✓ Type Safe    - Rust ensures correctness"
Enter
Sleep 3s
Screenshot examples/vhs/screenshots/arch-05-benefits.png
Clear

# Example workflow
Type "# Example: Flashing Workflow"
Enter
Sleep 1s
Enter
Type "1. User clicks 'Select Image'"
Enter
Type "   → Message::SelectImageClicked"
Enter
Sleep 1s
Enter
Type "2. Update triggers file dialog (Command)"
Enter
Type "   → Task::perform(select_file(), ...)"
Enter
Sleep 1s
Enter
Type "3. User selects file, dialog returns path"
Enter
Type "   → Message::ImageSelected(path)"
Enter
Sleep 1s
Enter
Type "4. Update stores path in state"
Enter
Type "   → self.selected_image = Some(path)"
Enter
Sleep 1s
Enter
Type "5. View re-renders with new state"
Enter
Type "   → Shows selected image details"
Enter
Sleep 4s
Screenshot examples/vhs/screenshots/arch-06-workflow.png
Clear

# Code organization
Type "# FlashKraft Code Organization"
Enter
Sleep 1s
Enter
Type "src/"
Enter
Type "├── model.rs           # State structures"
Enter
Type "├── message.rs         # Event definitions"
Enter
Type "├── update.rs          # State transitions"
Enter
Type "├── view.rs            # UI rendering"
Enter
Type "├── command.rs         # Async operations"
Enter
Type "└── main.rs            # Application setup"
Enter
Sleep 3s
Screenshot examples/vhs/screenshots/arch-07-organization.png
Clear

# See it in action
Type "# See The Elm Architecture in Action"
Enter
Sleep 1s
Enter
Type "# Run the examples:"
Enter
Type "cargo run --example basic_usage"
Enter
Type "cargo run --example state_machine"
Enter
Type "cargo run --example custom_theme"
Enter
Sleep 2s
Enter
Type "# Or run the full application:"
Enter
Type "cargo run"
Enter
Sleep 3s
Clear

# Resources
Type "# Learn More"
Enter
Sleep 1s
Enter
Type "📚 Documentation:"
Enter
Type "   - ELM_ARCHITECTURE.md"
Enter
Type "   - examples/README.md"
Enter
Type "   - Iced docs: docs.rs/iced"
Enter
Sleep 1s
Enter
Type "🔗 Resources:"
Enter
Type "   - guide.elm-lang.org/architecture"
Enter
Type "   - github.com/iced-rs/iced"
Enter
Sleep 3s
Clear

# Final message
Type "# The Elm Architecture = Simple + Scalable"
Enter
Sleep 1s
Enter
Type "Start simple, grow complex naturally."
Enter
Type "FlashKraft is built on these principles! ⚡"
Enter
Sleep 3s
