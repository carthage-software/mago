use ast::AstCommand;
use clap::builder::styling::AnsiColor;
use clap::builder::styling::Effects;
use clap::builder::Styles;
use clap::Parser;

use crate::commands::fix::FixCommand;
use crate::commands::format::FormatCommand;
use crate::commands::lint::LintCommand;

pub mod ast;
pub mod fix;
pub mod format;
pub mod lint;

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
    styles = CLAP_STYLING,
    // Ascii art by Todd Vargo (https://ascii.co.uk/art/fox)
    long_about = r#"
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
"#,
)]
pub enum FennecCommand {
    #[command(name = "lint")]
    Lint(LintCommand),
    #[command(name = "fix")]
    Fix(FixCommand),
    #[command(name = "format")]
    Format(FormatCommand),
    #[command(name = "ast")]
    Ast(AstCommand),
}
