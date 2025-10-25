# VHS tape for FlashKraft - The Elm Architecture Explanation
# This demonstrates the architecture and data flow of FlashKraft

Output examples/vhs/demo-architecture.gif
Set FontSize 14
Set Width 1400
Set Height 900
Set Theme "Dracula"
Set PlaybackSpeed 0.5

# Title Screen
Type "╔════════════════════════════════════════════════════════════╗"
Enter
Type "║         FlashKraft - The Elm Architecture                 ║"
Enter
Type "║         A Pure Functional Approach to GUI Apps            ║"
Enter
Type "╚════════════════════════════════════════════════════════════╝"
Enter
Sleep 3s

Enter
Type "Press any key to continue..."
Sleep 2s
Enter
Enter

# Section 1: What is The Elm Architecture?
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "1. What is The Elm Architecture?"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "The Elm Architecture (TEA) is a pattern for building interactive"
Enter
Type "applications with a simple, predictable state management flow."
Enter
Sleep 2s

Enter
Type "Core Components:"
Enter
Type "  1. Model     → Application State"
Enter
Type "  2. Message   → Events that trigger changes"
Enter
Type "  3. Update    → State transition logic"
Enter
Type "  4. View      → UI rendering based on state"
Enter
Sleep 3s

Enter
Enter
Screenshot examples/vhs/screenshots/arch-01-tea-intro.png
Sleep 1s

# Section 2: Data Flow Diagram
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "2. Data Flow - The Complete Cycle"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "┌─────────────────────────────────────────────────────┐"
Enter
Type "│              USER INTERFACE                         │"
Enter
Type "└─────────────────────────────────────────────────────┘"
Enter
Type "                      │"
Enter
Type "                      │ User Action (Click)"
Enter
Type "                      ▼"
Enter
Type "┌─────────────────────────────────────────────────────┐"
Enter
Type "│              MESSAGE                                │"
Enter
Type "│   (SelectImageClicked, FlashClicked, etc.)          │"
Enter
Type "└─────────────────────────────────────────────────────┘"
Enter
Type "                      │"
Enter
Type "                      ▼"
Enter
Type "┌─────────────────────────────────────────────────────┐"
Enter
Type "│              UPDATE                                 │"
Enter
Type "│   • Process message                                 │"
Enter
Type "│   • Update state                                    │"
Enter
Type "│   • Return Command (optional)                       │"
Enter
Type "└─────────────────────────────────────────────────────┘"
Enter
Type "                      │"
Enter
Type "          ┌───────────┴────────────┐"
Enter
Type "          ▼                        ▼"
Enter
Type "   ┌────────────┐          ┌──────────────┐"
Enter
Type "   │ NEW STATE  │          │   COMMAND    │"
Enter
Type "   └────────────┘          │ (Side Effect)│"
Enter
Type "          │                └──────────────┘"
Enter
Type "          │                        │"
Enter
Type "          ▼                        ▼"
Enter
Type "   ┌────────────┐          ┌──────────────┐"
Enter
Type "   │    VIEW    │          │  Async Task  │"
Enter
Type "   │  (Render)  │          │ (File Dialog)│"
Enter
Type "   └────────────┘          └──────────────┘"
Enter
Type "          │                        │"
Enter
Type "          ▼                        │"
Enter
Type "┌─────────────────────────────────────────────────────┐"
Enter
Type "│         UI UPDATED          ◄───────────────────────┘"
Enter
Type "└─────────────────────────────────────────────────────┘"
Enter
Type "                              Result → New Message"
Enter
Sleep 5s

Enter
Enter
Screenshot examples/vhs/screenshots/arch-02-data-flow.png
Sleep 2s

# Section 3: Model (State)
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "3. Model - The Application State"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

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
Type "    flash_status: FlashStatus,"
Enter
Type "    theme: Theme,"
Enter
Type "}"
Enter
Sleep 3s

Enter
Type "Key Principles:"
Enter
Type "  ✓ Immutable - State is never mutated directly"
Enter
Type "  ✓ Complete - Contains ALL information for UI"
Enter
Type "  ✓ Type-Safe - Invalid states are unrepresentable"
Enter
Sleep 3s

Enter
Enter
Screenshot examples/vhs/screenshots/arch-03-model.png
Sleep 1s

# Section 4: Message (Events)
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "4. Message - Events That Drive Changes"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "enum Message {"
Enter
Type "    // User Interactions"
Enter
Type "    SelectImageClicked,"
Enter
Type "    RefreshDrivesClicked,"
Enter
Type "    TargetDriveClicked(DriveInfo),"
Enter
Type "    FlashClicked,"
Enter
Type "    CancelFlash,"
Enter
Sleep 2s

Enter
Type "    // Async Results"
Enter
Type "    ImageSelected(Option<PathBuf>),"
Enter
Type "    DrivesRefreshed(Vec<DriveInfo>),"
Enter
Type "    FlashProgressUpdate(f32, u64, f32),"
Enter
Type "    FlashCompleted(Result<(), String>),"
Enter
Sleep 2s

Enter
Type "    // Theme & UI"
Enter
Type "    ThemeChanged(Theme),"
Enter
Type "    AnimationTick,"
Enter
Type "}"
Enter
Sleep 3s

Enter
Type "Messages are the ONLY way to change state!"
Enter
Sleep 2s

Enter
Enter
Screenshot examples/vhs/screenshots/arch-04-message.png
Sleep 1s

# Section 5: Update Logic
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "5. Update - The Heart of The Elm Architecture"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "fn update(&mut self, message: Message) -> Command<Message> {"
Enter
Type "    match message {"
Enter
Sleep 1s

Enter
Type "        Message::SelectImageClicked => {"
Enter
Type "            // Spawn async file dialog"
Enter
Type "            Command::perform("
Enter
Type "                select_image_file(),"
Enter
Type "                Message::ImageSelected"
Enter
Type "            )"
Enter
Type "        }"
Enter
Sleep 2s

Enter
Type "        Message::ImageSelected(Some(path)) => {"
Enter
Type "            // Update state with selected image"
Enter
Type "            self.selected_image = Some(ImageInfo::from(path));"
Enter
Type "            self.save_state(); // Persist"
Enter
Type "            Command::none()"
Enter
Type "        }"
Enter
Sleep 2s

Enter
Type "        Message::FlashClicked => {"
Enter
Type "            // Validate and start flash"
Enter
Type "            if self.can_flash() {"
Enter
Type "                self.flash_status = FlashStatus::Flashing;"
Enter
Type "                Command::perform("
Enter
Type "                    flash_operation(),"
Enter
Type "                    Message::FlashCompleted"
Enter
Type "                )"
Enter
Type "            } else {"
Enter
Type "                Command::none()"
Enter
Type "            }"
Enter
Type "        }"
Enter
Sleep 3s

Enter
Type "        // ... handle other messages"
Enter
Type "    }"
Enter
Type "}"
Enter
Sleep 2s

Enter
Enter
Screenshot examples/vhs/screenshots/arch-05-update.png
Sleep 1s

# Section 6: View Logic
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "6. View - Declarative UI Rendering"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "fn view(&self) -> Element<'_, Message> {"
Enter
Type "    match self.flash_status {"
Enter
Sleep 1s

Enter
Type "        FlashStatus::Idle => {"
Enter
Type "            // Show main selection UI"
Enter
Type "            view_main_screen(self)"
Enter
Type "        }"
Enter
Sleep 1s

Enter
Type "        FlashStatus::Flashing => {"
Enter
Type "            // Show progress bar"
Enter
Type "            view_flashing_screen(self)"
Enter
Type "        }"
Enter
Sleep 1s

Enter
Type "        FlashStatus::Complete => {"
Enter
Type "            // Show success message"
Enter
Type "            view_success_screen()"
Enter
Type "        }"
Enter
Sleep 1s

Enter
Type "        FlashStatus::Error(msg) => {"
Enter
Type "            // Show error message"
Enter
Type "            view_error_screen(msg)"
Enter
Type "        }"
Enter
Type "    }"
Enter
Type "}"
Enter
Sleep 3s

Enter
Type "View is PURE: Same state → Same UI"
Enter
Sleep 2s

Enter
Enter
Screenshot examples/vhs/screenshots/arch-06-view.png
Sleep 1s

# Section 7: Complete Example Flow
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "7. Complete Flow Example - User Selects Image"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "Step 1: User clicks 'Select Image' button"
Enter
Type "  UI → Message::SelectImageClicked → Update"
Enter
Sleep 2s

Enter
Type "Step 2: Update spawns async command"
Enter
Type "  Command::perform(select_image_file(), ...)"
Enter
Sleep 2s

Enter
Type "Step 3: File dialog opens (async operation)"
Enter
Type "  User selects: ubuntu-22.04-desktop-amd64.iso"
Enter
Sleep 2s

Enter
Type "Step 4: Command completes → New Message"
Enter
Type "  Result → Message::ImageSelected(Some(path))"
Enter
Sleep 2s

Enter
Type "Step 5: Update processes result"
Enter
Type "  self.selected_image = Some(ImageInfo {...})"
Enter
Sleep 2s

Enter
Type "Step 6: View re-renders automatically"
Enter
Type "  UI now shows: ✓ ubuntu-22.04-desktop-amd64.iso (3.8 GB)"
Enter
Sleep 3s

Enter
Type "ALL state changes flow through this cycle!"
Enter
Sleep 2s

Enter
Enter
Screenshot examples/vhs/screenshots/arch-07-flow-example.png
Sleep 1s

# Section 8: Benefits
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "8. Why The Elm Architecture?"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "✓ PREDICTABILITY"
Enter
Type "  All state changes flow through update()"
Enter
Type "  Easy to trace and understand behavior"
Enter
Sleep 2s

Enter
Type "✓ TESTABILITY"
Enter
Type "  Pure functions are trivial to test"
Enter
Type "  No mocking needed for update/view logic"
Enter
Sleep 2s

Enter
Type "✓ DEBUGGABILITY"
Enter
Type "  Log every message to see exact flow"
Enter
Type "  Time-travel debugging possible"
Enter
Sleep 2s

Enter
Type "✓ MAINTAINABILITY"
Enter
Type "  Clear separation of concerns"
Enter
Type "  Easy to add new features"
Enter
Sleep 2s

Enter
Type "✓ TYPE SAFETY"
Enter
Type "  Rust ensures correctness at compile time"
Enter
Type "  Invalid states are unrepresentable"
Enter
Sleep 3s

Enter
Enter
Screenshot examples/vhs/screenshots/arch-08-benefits.png
Sleep 1s

# Section 9: FlashKraft Architecture
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "9. FlashKraft Project Structure"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "src/"
Enter
Type "├── core/              # Elm Architecture Core"
Enter
Type "│   ├── state.rs       # Model (Application State)"
Enter
Type "│   ├── message.rs     # Message definitions"
Enter
Type "│   ├── update.rs      # Update logic"
Enter
Type "│   ├── storage.rs     # Persistent storage"
Enter
Type "│   └── commands/      # Async commands"
Enter
Sleep 2s

Enter
Type "├── domain/            # Business Entities"
Enter
Type "│   ├── drive_info.rs  # Drive information"
Enter
Type "│   └── image_info.rs  # Image information"
Enter
Sleep 2s

Enter
Type "├── components/        # UI Components"
Enter
Type "│   ├── header.rs"
Enter
Type "│   ├── selection_panels.rs"
Enter
Type "│   ├── device_selector.rs"
Enter
Type "│   ├── animated_progress.rs"
Enter
Type "│   └── theme_selector.rs"
Enter
Sleep 2s

Enter
Type "├── view.rs            # View orchestration"
Enter
Type "└── main.rs            # Entry point"
Enter
Sleep 3s

Enter
Type "26 modules | 4 layers | Clean architecture"
Enter
Sleep 2s

Enter
Enter
Screenshot examples/vhs/screenshots/arch-09-structure.png
Sleep 1s

# Section 10: Summary
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type "10. Summary - The Elm Architecture in Action"
Enter
Type "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 2s

Enter
Type "The Elm Architecture provides:"
Enter
Sleep 1s

Enter
Type "  🎯 Single source of truth (Model)"
Enter
Type "  📬 Explicit events (Messages)"
Enter
Type "  🔄 Pure state transitions (Update)"
Enter
Type "  🎨 Declarative rendering (View)"
Enter
Type "  ⚡ Safe side effects (Commands)"
Enter
Sleep 3s

Enter
Type "Result: Predictable, testable, maintainable code!"
Enter
Sleep 2s

Enter
Enter
Type "Learn more:"
Enter
Type "  • docs/ELM_ARCHITECTURE.md"
Enter
Type "  • examples/basic_usage.rs"
Enter
Type "  • https://guide.elm-lang.org/architecture/"
Enter
Sleep 3s

Enter
Enter
Screenshot examples/vhs/screenshots/arch-10-summary.png
Sleep 2s

Enter
Enter
Type "╔════════════════════════════════════════════════════════════╗"
Enter
Type "║              Thanks for watching!                         ║"
Enter
Type "║         FlashKraft - Built with Rust + Iced + TEA        ║"
Enter
Type "╚════════════════════════════════════════════════════════════╝"
Enter
Sleep 3s
