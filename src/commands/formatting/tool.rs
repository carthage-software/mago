use mago_database::ReadDatabase;
use mago_database::file::File;
use mago_orchestrator::Orchestrator;
use mago_orchestrator::OrchestratorError;
use mago_orchestrator::service::format::FileFormatStatus;
use mago_orchestrator::service::format::FormatResult;
use mago_orchestrator::service::format::FormatService;
use mago_orchestrator::service::twig_format::TwigFormatService;
use mago_twig_formatter::settings::FormatSettings;

use crate::config::Configuration;

#[derive(Debug, Clone, Copy)]
pub(crate) enum FormatTool {
    Php,
    Twig,
}

impl FormatTool {
    pub fn enabled(self, configuration: &Configuration) -> bool {
        match self {
            Self::Php => true,
            Self::Twig => configuration
                .twig
                .as_ref()
                .and_then(|twig| twig.formatter.as_ref())
                .is_none_or(|formatter| formatter.enabled),
        }
    }

    pub fn configure<'cfg>(self, orchestrator: &mut Orchestrator<'cfg>, configuration: &'cfg Configuration) {
        match self {
            Self::Php => orchestrator.add_exclude_patterns(configuration.formatter.excludes.iter()),
            Self::Twig => {
                orchestrator.config.extensions = vec!["twig"];
                if let Some(formatter) = configuration.twig.as_ref().and_then(|twig| twig.formatter.as_ref()) {
                    orchestrator.add_exclude_patterns(formatter.excludes.iter());
                }
            }
        }
    }

    pub(super) fn service(
        self,
        orchestrator: &Orchestrator<'_>,
        database: ReadDatabase,
        configuration: &Configuration,
    ) -> FormatEngine {
        match self {
            Self::Php => FormatEngine::Php(orchestrator.get_format_service(database)),
            Self::Twig => {
                let settings = configuration
                    .twig
                    .as_ref()
                    .and_then(|twig| twig.formatter.as_ref())
                    .map_or_else(FormatSettings::default, |formatter| formatter.settings);

                FormatEngine::Twig(orchestrator.get_twig_format_service(database, settings))
            }
        }
    }
}

pub(super) enum FormatEngine {
    Php(FormatService),
    Twig(TwigFormatService),
}

impl FormatEngine {
    pub fn run(self) -> Result<FormatResult, OrchestratorError> {
        match self {
            Self::Php(service) => service.run(),
            Self::Twig(service) => service.run(),
        }
    }

    pub fn format_file(self, file: &File) -> Result<FileFormatStatus, OrchestratorError> {
        match self {
            Self::Php(service) => service.format_file(file),
            Self::Twig(service) => service.format_file(file),
        }
    }
}
