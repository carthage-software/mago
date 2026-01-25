/// Choice for colorizing output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ColorChoice {
    /// Automatically detect whether colors should be used based on TTY detection.
    #[default]
    Auto,
    /// Always use colors, regardless of TTY status.
    Always,
    /// Never use colors.
    Never,
}

impl ColorChoice {
    /// Determine if colors should be used based on this choice and whether output is a TTY.
    ///
    /// # Arguments
    ///
    /// * `is_tty` - Whether the output stream is connected to a terminal
    ///
    /// # Returns
    ///
    /// `true` if colors should be used, `false` otherwise
    #[must_use]
    pub fn should_use_colors(self, is_tty: bool) -> bool {
        // Respect NO_COLOR environment variable (https://no-color.org/)
        // If NO_COLOR exists and is not "0", disable colors
        if std::env::var_os("NO_COLOR").is_some_and(|value| value != "0") {
            return false;
        }

        match self {
            ColorChoice::Auto => is_tty,
            ColorChoice::Always => true,
            ColorChoice::Never => false,
        }
    }
}
