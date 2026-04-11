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
        // FORCE_COLOR takes precedence (https://force-color.org/).
        // Any non-empty value forces colors, except "0" which explicitly disables them.
        if let Some(value) = std::env::var_os("FORCE_COLOR")
            && !value.is_empty()
        {
            return value != "0";
        }

        // NO_COLOR (https://no-color.org/): when set to any non-empty value,
        // colors must be suppressed. An empty value has no effect.
        if std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty()) {
            return false;
        }

        match self {
            ColorChoice::Auto => is_tty,
            ColorChoice::Always => true,
            ColorChoice::Never => false,
        }
    }
}
