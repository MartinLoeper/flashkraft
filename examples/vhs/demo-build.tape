# VHS tape for FlashKraft - Build and Setup Demo
# This demonstrates building and setting up FlashKraft

Output examples/vhs/demo-build.gif
Set FontSize 16
Set Width 1400
Set Height 900
Set Theme "CatppuccinMocha"
Set PlaybackSpeed 1.0

# Title
Type "# FlashKraft - Build & Setup"
Enter
Sleep 1s
Type "# Modern OS Image Writer in Rust"
Enter
Sleep 2s
Clear

# Show project structure
Type "# Project Structure"
Enter
Sleep 1s
Type "ls -la"
Enter
Sleep 2s
Screenshot examples/vhs/screenshots/build-01-project-files.png
Sleep 2s
Clear

# Show Cargo.toml
Type "# Dependencies"
Enter
Sleep 1s
Type "cat Cargo.toml"
Enter
Sleep 3s
Screenshot examples/vhs/screenshots/build-02-dependencies.png
Sleep 2s
Clear

# Check Rust version
Type "# Prerequisites"
Enter
Sleep 1s
Type "rustc --version"
Enter
Sleep 1s
Type "cargo --version"
Enter
Sleep 2s
Screenshot examples/vhs/screenshots/build-03-rust-version.png
Sleep 2s
Clear

# Run cargo check
Type "# Check Code"
Enter
Sleep 1s
Type "cargo check"
Enter
Sleep 5s
Screenshot examples/vhs/screenshots/build-04-cargo-check.png
Sleep 2s
Clear

# Run tests
Type "# Run Tests"
Enter
Sleep 1s
Type "cargo test --lib"
Enter
Sleep 6s
Screenshot examples/vhs/screenshots/build-05-tests-passing.png
Sleep 2s
Clear

# Build release
Type "# Build Release Version"
Enter
Sleep 1s
Type "cargo build --release"
Enter
Sleep 8s
Screenshot examples/vhs/screenshots/build-06-release-build.png
Sleep 2s
Clear

# Show binary size
Type "# Build Complete!"
Enter
Sleep 1s
Type "ls -lh target/release/flashkraft"
Enter
Sleep 2s
Screenshot examples/vhs/screenshots/build-07-binary-info.png
Sleep 2s
Clear

# Run the application
Type "# Run FlashKraft"
Enter
Sleep 1s
Type "cargo run --release"
Enter
Sleep 5s
Screenshot examples/vhs/screenshots/build-08-app-running.png
Sleep 3s

# Stop the app
Ctrl+C
Sleep 1s
Clear

# Show examples
Type "# Available Examples"
Enter
Sleep 1s
Type "ls examples/"
Enter
Sleep 2s
Screenshot examples/vhs/screenshots/build-09-examples-list.png
Sleep 1s
Enter
Type "# Run an example:"
Enter
Type "cargo run --example basic_usage"
Enter
Sleep 4s
Screenshot examples/vhs/screenshots/build-10-example-running.png
Sleep 2s

# Stop
Ctrl+C
Sleep 1s
Clear

# Generate docs
Type "# Generate Documentation"
Enter
Sleep 1s
Type "cargo doc --no-deps --open"
Enter
Sleep 3s
Screenshot examples/vhs/screenshots/build-11-docs-generated.png
Sleep 2s
Ctrl+C
Clear

# Final message
Type "# Setup Complete! ✓"
Enter
Sleep 1s
Enter
Type "Ready to flash OS images with style!"
Enter
Sleep 3s
