//! Ghost-text command suggestions for zsh from a local model that reads your terminal.

pub mod context;
pub mod daemon;
pub mod doctor;
pub mod history;
#[cfg(target_os = "macos")]
pub mod menubar;
pub mod model;
pub mod paths;
pub mod screen;
pub mod service;
pub mod setup;
pub mod stats;
pub mod status;
