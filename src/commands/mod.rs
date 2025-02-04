use clap::builder::styling::AnsiColor;
use clap::builder::styling::Effects;
use clap::builder::Styles;
use clap::Parser;

use crate::commands::ast::AstCommand;
use crate::commands::find::FindCommand;
use crate::commands::fix::FixCommand;
use crate::commands::format::FormatCommand;
use crate::commands::lint::LintCommand;
use crate::commands::self_update::SelfUpdateCommand;

pub mod ast;
pub mod find;
pub mod fix;
pub mod format;
pub mod lint;
pub mod self_update;

pub const CLAP_STYLING: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default())
    .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
    .valid(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD));

#[derive(Parser, Debug)]
#[command(
    version,
    author,
    styles = CLAP_STYLING,
    about = "Mago: The Oxidized PHP Toolchain

Empower your PHP projects with the magic of Mago. Analyze, lint, format, and refactor
with speed, precision, and elegance.",
    long_about = r#"
Welcome to Mago!

Mago is your all-in-one PHP toolkit, designed to make your development experience faster, easier, and a bit magical.

Here’s what makes Mago special:

- Lint with Precision: Spot and fix issues in your code with powerful linting capabilities.
- Effortless Formatting: Standardize your codebase with beautiful and consistent formatting.
- Smarter Refactoring: Automate repetitive tasks and ensure semantic correctness.
- Blazing Fast Performance: Built with Rust, Mago is optimized for speed and reliability.

🛠  Start transforming your PHP workflow today! Explore the commands below to unleash the full power of Mago 💻
"#)]
pub enum MagoCommand {
    #[command(name = "ast")]
    Ast(AstCommand),
    #[command(name = "lint")]
    Lint(LintCommand),
    #[command(name = "fix")]
    Fix(FixCommand),
    #[command(name = "format")]
    Format(FormatCommand),
    #[command(name = "find")]
    Find(FindCommand),
    #[command(name = "self-update")]
    SelfUpdate(SelfUpdateCommand),
}
