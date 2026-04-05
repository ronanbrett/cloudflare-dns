/// UI components and application state management.
///
/// This module contains all TUI-related code including:
/// - Application state and props
/// - Event handling
/// - Status messages
/// - UI components
/// - Theme/colors
pub mod app;
pub mod colors;
pub mod components;
pub mod constants;
pub mod hooks;
pub mod state;
pub mod status;
pub mod theme;

// Re-export commonly used types
pub use app::run_app;
#[allow(unused_imports)]
pub use state::{AppState, AppView};
