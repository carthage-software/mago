#![allow(unknown_lints)]
#![allow(clippy::literal_string_with_formatting_args)]

use std::sync::LazyLock;

use indicatif::MultiProgress;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;

/// A global multi-progress bar that allows managing multiple progress bars concurrently.
pub static GLOBAL_PROGRESS_MANAGER: LazyLock<MultiProgress> = LazyLock::new(MultiProgress::new);

/// Creates a new progress bar with the specified length and theme.
///
/// # Arguments
///
/// * `length` - The total length of the progress bar, representing the total units of work.
/// * `theme` - The theme of the progress bar.
///
/// # Returns
///
/// A `ProgressBar` that is styled and ready to use.
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

/// Removes the specified progress bar from the global multi-progress manager.
///
/// # Arguments
///
/// * `progress_bar` - The progress bar to remove.
pub fn remove_progress_bar(progress_bar: ProgressBar) {
    progress_bar.finish_and_clear();

    GLOBAL_PROGRESS_MANAGER.remove(&progress_bar);
}

/// Represents different visual themes for the progress bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProgressBarTheme {
    Red,
    Yellow,
    Green,
    Blue,
    Magenta,
    Cyan,
}

impl ProgressBarTheme {
    /// Returns the template string for the selected theme, defining the layout and appearance of the progress bar.
    pub fn template(&self) -> &'static str {
        match self {
            Self::Red => "{spinner} {prefix:<16.bold}▕{wide_bar:.red}▏{pos:>6}/{len}▕  {percent:>3}%▕  ETA: {eta_precise}▕  Elapsed: {elapsed_precise}",
            Self::Yellow => "{spinner} {prefix:<16.bold}▕{wide_bar:.yellow}▏{pos:>6}/{len}▕  {percent:>3}%▕  ETA: {eta_precise}▕  Elapsed: {elapsed_precise}",
            Self::Green => "{spinner} {prefix:<16.bold}▕{wide_bar:.green}▏{pos:>6}/{len}▕  {percent:>3}%▕  ETA: {eta_precise}▕  Elapsed: {elapsed_precise}",
            Self::Blue => "{spinner} {prefix:<16.bold}▕{wide_bar:.blue}▏{pos:>6}/{len}▕  {percent:>3}%▕  ETA: {eta_precise}▕  Elapsed: {elapsed_precise}",
            Self::Magenta => "{spinner} {prefix:<16.bold}▕{wide_bar:.magenta}▏{pos:>6}/{len}▕  {percent:>3}%▕  ETA: {eta_precise}▕  Elapsed: {elapsed_precise}",
            Self::Cyan => "{spinner} {prefix:<16.bold}▕{wide_bar:.cyan}▏{pos:>6}/{len}▕  {percent:>3}%▕  ETA: {eta_precise}▕  Elapsed: {elapsed_precise}",
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
