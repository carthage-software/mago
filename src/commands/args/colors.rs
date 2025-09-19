use clap::Parser;

/// Defines command-line options for controlling colored output.
#[derive(Parser, Debug, Clone)]
pub struct ColorArgs {
    /// Do not use colors in the output.
    #[arg(long, default_value_t = false, alias = "no-colors")]
    pub no_color: bool,
}

impl ColorArgs {
    pub(crate) fn should_use_colors(&self) -> bool {
        !self.no_color
    }
}
