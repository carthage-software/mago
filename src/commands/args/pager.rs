use clap::Parser;

use crate::config::Configuration;

/// Defines command-line options for pager functionality.
#[derive(Parser, Debug, Clone)]
pub struct PagerArgs {
    /// Control whether to use a pager for long output.
    ///
    /// A pager (like 'less') allows you to scroll through long output instead of
    /// having it all scroll past in the terminal. Use --pager=true to force enable,
    /// --pager=false to disable, or just --pager to enable with default settings.
    #[arg(long, num_args(0..=1), default_missing_value = "true")]
    pub pager: Option<bool>,
}

impl PagerArgs {
    pub(crate) fn should_use_pager(&self, configuration: &Configuration) -> bool {
        match self.pager {
            Some(true) => {
                #[cfg(not(unix))]
                {
                    tracing::warn!("Pager is only supported on unix-like systems. falling back to no pager.");
                    false
                }

                #[cfg(unix)]
                true
            }
            Some(false) => false,
            None => {
                // If this is true on non-unix systems, it would have been reported in
                // the main function during initialization.
                #[cfg(not(unix))]
                {
                    false
                }

                #[cfg(unix)]
                {
                    configuration.use_pager
                }
            }
        }
    }
}
