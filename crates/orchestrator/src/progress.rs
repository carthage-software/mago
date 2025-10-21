//! Progress bar utilities for long-running operations.
//!
//! This module provides a unified way to create and manage progress bars across all
//! orchestrator services. It uses a global [`MultiProgress`] manager to coordinate
//! multiple concurrent progress bars without visual conflicts.
//!
//! # Features
//!
//! - **Themed Progress Bars**: Six different visual themes with distinct colors and animations
//! - **Concurrent Management**: Global manager ensures multiple progress bars render correctly
//! - **Automatic Cleanup**: Progress bars are automatically removed when finished
//! - **Rich Information**: Displays current/total, percentage, ETA, and elapsed time
//!
//! # Themes
//!
//! Different services use different themes by convention:
//! - **Blue**: Compilation/indexing phase (scanner, parser)
//! - **Green**: Main analysis tasks
//! - **Magenta**: Stateless operations (linting, formatting)
//! - **Red**: Error-prone or critical operations
//! - **Yellow**: Warning-level operations
//! - **Cyan**: Information gathering

#![allow(unknown_lints)]
#![allow(clippy::literal_string_with_formatting_args)]
#![allow(dead_code)]

use std::sync::LazyLock;

use indicatif::MultiProgress;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;

/// Global progress bar manager for coordinating concurrent progress bars.
///
/// This singleton manages all progress bars in the application, ensuring they
/// render correctly when multiple services run concurrently. All progress bars
/// created via [`create_progress_bar`] are automatically registered with this manager.
///
/// The manager handles:
/// - Preventing visual conflicts between concurrent progress bars
/// - Maintaining proper terminal output ordering
/// - Cleanup of finished progress bars
pub static GLOBAL_PROGRESS_MANAGER: LazyLock<MultiProgress> = LazyLock::new(MultiProgress::new);

/// Creates a new styled progress bar and registers it with the global manager.
///
/// The progress bar is automatically registered with the [`GLOBAL_PROGRESS_MANAGER`] to
/// ensure proper rendering when multiple progress bars are active concurrently.
///
/// # Arguments
///
/// * `length` - The total number of units of work (e.g., number of files to process)
/// * `prefix` - A label to display before the progress bar (e.g., "Linting", "📚 Compiling")
/// * `theme` - The visual theme determining colors and animation style
///
/// # Returns
///
/// A fully configured [`ProgressBar`] ready to use. Call `inc(1)` to increment progress,
/// and [`remove_progress_bar`] when done.
pub fn create_progress_bar(length: usize, prefix: &'static str, theme: ProgressBarTheme) -> ProgressBar {
    let pb = GLOBAL_PROGRESS_MANAGER.add(ProgressBar::new(length as u64));
    pb.set_style(
        ProgressStyle::with_template(theme.template())
            .unwrap()
            .progress_chars(theme.progress_chars())
            .tick_chars(theme.tick_chars()),
    );

    pb.set_prefix(prefix);
    pb
}

/// Removes a progress bar from display and cleans up its resources.
///
/// This function finishes the progress bar (clearing it from the terminal) and
/// removes it from the global progress manager. Always call this when you're done
/// with a progress bar to prevent visual artifacts.
///
/// # Arguments
///
/// * `progress_bar` - The progress bar to remove (obtained from [`create_progress_bar`])
pub fn remove_progress_bar(progress_bar: ProgressBar) {
    progress_bar.finish_and_clear();

    GLOBAL_PROGRESS_MANAGER.remove(&progress_bar);
}

/// Visual themes for progress bars.
///
/// Each theme has a distinct color scheme, progress characters, and spinner animation.
/// Different services use different themes by convention to make it easier to identify
/// which operation is running when multiple progress bars are displayed concurrently.
///
/// # Theme Conventions
///
/// - **Blue**: Compilation or indexing phase (e.g., scanning files, building codebase metadata)
/// - **Green**: Main analysis or processing tasks
/// - **Magenta**: Stateless operations like linting or formatting
/// - **Red**: Error-prone or critical operations
/// - **Yellow**: Warning-level operations
/// - **Cyan**: Information gathering or auxiliary tasks
///
/// # Visual Characteristics
///
/// Each theme uses unique:
/// - **Colors**: ANSI terminal colors for the progress bar
/// - **Progress Characters**: Unicode block characters showing completion
/// - **Tick Characters**: Braille patterns for the spinner animation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ProgressBarTheme {
    /// Red theme - for error-prone or critical operations
    Red,
    /// Yellow theme - for warning-level operations
    Yellow,
    /// Green theme - for main analysis or processing tasks
    Green,
    /// Blue theme - for compilation or indexing phases
    Blue,
    /// Magenta theme - for stateless operations (linting, formatting)
    Magenta,
    /// Cyan theme - for information gathering or auxiliary tasks
    Cyan,
}

impl ProgressBarTheme {
    /// Returns the template string for the selected theme, defining the layout and appearance of the progress bar.
    pub fn template(&self) -> &'static str {
        match self {
            Self::Red => {
                "{spinner} {prefix:<16.bold}▕{wide_bar:.red}▏{pos:>6}/{len}▕  {percent:>3}%▕  ETA: {eta_precise}▕  Elapsed: {elapsed_precise}"
            }
            Self::Yellow => {
                "{spinner} {prefix:<16.bold}▕{wide_bar:.yellow}▏{pos:>6}/{len}▕  {percent:>3}%▕  ETA: {eta_precise}▕  Elapsed: {elapsed_precise}"
            }
            Self::Green => {
                "{spinner} {prefix:<16.bold}▕{wide_bar:.green}▏{pos:>6}/{len}▕  {percent:>3}%▕  ETA: {eta_precise}▕  Elapsed: {elapsed_precise}"
            }
            Self::Blue => {
                "{spinner} {prefix:<16.bold}▕{wide_bar:.blue}▏{pos:>6}/{len}▕  {percent:>3}%▕  ETA: {eta_precise}▕  Elapsed: {elapsed_precise}"
            }
            Self::Magenta => {
                "{spinner} {prefix:<16.bold}▕{wide_bar:.magenta}▏{pos:>6}/{len}▕  {percent:>3}%▕  ETA: {eta_precise}▕  Elapsed: {elapsed_precise}"
            }
            Self::Cyan => {
                "{spinner} {prefix:<16.bold}▕{wide_bar:.cyan}▏{pos:>6}/{len}▕  {percent:>3}%▕  ETA: {eta_precise}▕  Elapsed: {elapsed_precise}"
            }
        }
    }

    /// Returns the characters used to represent the progress of the bar.
    pub fn progress_chars(&self) -> &'static str {
        match self {
            ProgressBarTheme::Red => "█░ ",
            ProgressBarTheme::Yellow => "█▉▊▋▌▍▎▏░ ",
            ProgressBarTheme::Green => "█▇▆▅▄▃▂▁░ ",
            ProgressBarTheme::Blue => "█▓▒░░ ",
            ProgressBarTheme::Magenta => "█▛▌▖░ ",
            ProgressBarTheme::Cyan => "█▉▊▋▌▍▎▏░ ",
        }
    }

    /// Returns the characters used to animate the spinner/ticker in the progress bar.
    pub fn tick_chars(&self) -> &'static str {
        match self {
            ProgressBarTheme::Red => "⠁⠂⠄⡀⢀⠠⠐⠈ ",
            ProgressBarTheme::Yellow => "⢀⠠⠐⠈⠁⠂⠄⡀ ",
            ProgressBarTheme::Green => "⠄⡀⢀⠠⠐⠈⠁⠂ ",
            ProgressBarTheme::Blue => "⡀⢀⠠⠐⠈⠁⠂⠄ ",
            ProgressBarTheme::Magenta => "⠐⠈⠁⠂⠄⡀⢀⠠ ",
            ProgressBarTheme::Cyan => "⠠⠐⠈⠁⠂⠄⡀⢀ ",
        }
    }
}
