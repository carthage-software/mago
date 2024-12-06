use clap::Parser;

use fennec_interner::ThreadedInterner;
use fennec_service::config::Configuration;
use fennec_service::formatter::FormatterService;
use fennec_service::source::SourceService;

use crate::utils::bail;

#[derive(Parser, Debug)]
#[command(
    name = "format",
    aliases = ["fmt"],
    about = "Format source files",
    long_about = r#"
Format source files.

This command will format source files according to the rules defined in the configuration file.
"#
)]
pub struct FormatCommand {
    #[arg(long, short = 'w', help = "The width of the printed source code", value_name = "WIDTH")]
    pub print_width: Option<usize>,
}

pub async fn execute(command: FormatCommand, mut configuration: Configuration) -> i32 {
    let interner = ThreadedInterner::new();

    let source_service = SourceService::new(interner.clone(), configuration.source);
    let source_manager = source_service.load().await.unwrap_or_else(bail);

    if let Some(width) = command.print_width {
        configuration.format.print_width = Some(width);
    }

    let service = FormatterService::new(configuration.format, interner.clone(), source_manager.clone());

    let count = service.run().await.unwrap_or_else(bail);

    fennec_feedback::info!("formatted {} source files successfully", count);

    0
}
