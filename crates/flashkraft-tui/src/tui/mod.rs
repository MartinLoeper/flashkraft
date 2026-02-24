//! TUI module — groups all Ratatui front-end components.
//!
//! | Submodule        | Responsibility                                          |
//! |------------------|---------------------------------------------------------|
//! | `app`            | Application state machine & channel polling             |
//! | `events`         | Keyboard event → state-transition mapping               |
//! | `flash_runner`   | Tokio task that drives the privileged flash child       |
//! | `ui`             | All ratatui `Frame` rendering (one function per screen) |

pub mod app;
pub mod events;
pub mod flash_runner;
pub mod ui;
