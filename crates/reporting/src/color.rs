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
    pub fn should_use_colors(self, is_tty: bool) -> bool {
        match self {
            ColorChoice::Auto => is_tty,
            ColorChoice::Always => true,
            ColorChoice::Never => false,
        }
    }
}
