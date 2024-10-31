use clap::arg;
use clap::builder::styling::AnsiColor;
use clap::builder::styling::Effects;
use clap::builder::Styles;
use clap::Command;

use fennec_config::Configuration;
use fennec_fixer::SafetyClassification;
use fennec_interner::ThreadedInterner;
use fennec_reporting::reporter::*;
use fennec_source::SourceManager;

use crate::command::fix;
use crate::command::lint;
use crate::utils::error::bail;
use crate::utils::runner::run_with_configuration;

mod command;
mod utils;

pub const CLAP_STYLING: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default())
    .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
    .valid(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD));

/// Ascii art by Todd Vargo (https://ascii.co.uk/art/fox)
pub const LONG_ABOUT: &str = r#"
--------------------------------------------------------------------------
  /\   /\            |
 //\\_//\\     ____  | Fennec 🦊 is an all-in-one, oxidized PHP toolchain,
 \_     _/    /   /  | built to handle everything from static analysis and
  / * * \    /^^^]   | refactoring to full project management.
  \_\O/_/    [   ]   |
   /   \_    [   /   |
   \     \_  /  /    |
    [ [ /  \/ _/     | https://carthage.software/fennec
   _[ [ \  /_/       |
--------------------------------------------------------------------------
"#;

pub fn main() {
    let fennec = clap::command!("fennec")
        .bin_name("fennec")
        .styles(CLAP_STYLING)
        .subcommand_required(true)
        .long_about(LONG_ABOUT)
        // fennec lint [-f --fixable]
        .subcommand(
            Command::new("lint")
                .about("Lint the project according to the `fennec.toml` configuration or default settings")
                .long_about(r#"
                    Lint the project according to the `fennec.toml` configuration or default settings.

                    This command analyzes the project's source code and highlights issues based on the defined linting rules.

                    If `fennec.toml` is not found, the default configuration is used. The command outputs the issues found in the project."
                "#)
                .arg(arg!(-o --"only-fixable" "Only show fixable issues"))
        )
        // fennec fix [-u --unsafe] [-p --potentially-unsafe] [-d --dry-run]
        .subcommand(
            Command::new("fix")
                .about("Fix lint issues identified during the linting process")
                .long_about(r#"
                    Fix lint issues identified during the linting process.

                    Automatically applies fixes where possible, based on the rules in the `fennec.toml` or the default settings.
                "#)
                .arg(arg!(-u --unsafe "Apply modifications that are marked as unsafe, including potentially unsafe modifications"))
                .arg(arg!(-p --"potentially-unsafe" "Apply modifications that are marked as potentially unsafe"))
                .arg(arg!(-d --"dry-run" "Run the command without writing any changes to disk"))

        )
    ;

    match fennec.get_matches().subcommand() {
        Some(("lint", matches)) => {
            let only_fixable = matches.get_flag("only-fixable");

            run_with_configuration(|configuration: Configuration| async {
                let interner = ThreadedInterner::new();
                let source_manager = SourceManager::build(&interner, &configuration.source).await.unwrap_or_else(bail);

                let reporter = Reporter::new(source_manager.clone());

                lint::execute(configuration, interner, source_manager, reporter, only_fixable).await
            });
        }
        Some(("fix", matches)) => {
            let dry_run = matches.get_flag("dry-run");
            let safety_classification = if matches.get_flag("unsafe") {
                SafetyClassification::Unsafe
            } else if matches.get_flag("potentially-unsafe") {
                SafetyClassification::PotentiallyUnsafe
            } else {
                SafetyClassification::Safe
            };

            run_with_configuration(|configuration: Configuration| async {
                let interner = ThreadedInterner::new();
                let source_manager = SourceManager::build(&interner, &configuration.source).await.unwrap_or_else(bail);

                fix::execute(configuration, interner, source_manager, safety_classification, dry_run).await
            });
        }
        _ => unreachable!("clap should ensure we don't get here"),
    }
}
