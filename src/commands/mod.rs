use clap::builder::styling::AnsiColor;
use clap::builder::styling::Effects;
use clap::builder::Styles;
use clap::Parser;

use crate::commands::ast::AstCommand;
use crate::commands::fix::FixCommand;
use crate::commands::format::FormatCommand;
use crate::commands::lint::LintCommand;
use crate::commands::self_update::SelfUpdateCommand;

pub mod ast;
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
    about = "the ultimate toolkit for PHP developers – analyze, fix, and refactor your code with style",
    long_about = r#"
----------------------------------------------------------------------------------------------
  Welcome to Mago – the ultimate PHP toolkit, reimagined for the modern developer.

  Mago isn’t just a toolchain; it’s your secret weapon for taming PHP projects of any size.
  Whether you're diving into a legacy codebase or crafting something cutting-edge,
  Mago equips you with the tools to:

  🚀 Analyze, lint, and fix your code with unmatched speed and precision.
  🎨 Format your PHP effortlessly for consistent, beautiful code.
  🔍 Explore your code with powerful AST visualization.
  🛠 Refactor smarter, not harder, with automated fixes.
  🌐 Stay ahead with easy updates and blazing-fast performance.

  Designed to make PHP development faster, easier, and just a bit magical.

  Start transforming your workflow today. Learn more at: https://carthage.software/mago
  ----------------------------------------------------------------------------------------------
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
    #[command(name = "self-update")]
    SelfUpdate(SelfUpdateCommand),
}
