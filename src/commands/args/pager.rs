use clap::Parser;

use crate::config::Configuration;

/// Defines command-line options for pager functionality.
#[derive(Parser, Debug, Clone)]
pub struct PagerArgs {
    /// Use a pager when printing output.
    #[arg(
        long,
        help = "Use a pager when printing output",
        num_args(0..=1),
        default_missing_value = "true",
    )]
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
