use fennec_config::Configuration;
use fennec_interner::ThreadedInterner;
use fennec_reporting::reporter::Reporter;
use fennec_service::linter::LintService;
use fennec_source::SourceManager;

use crate::utils::error::bail;

pub async fn execute(
    configuration: Configuration,
    interner: ThreadedInterner,
    source_manager: SourceManager,
    reporter: Reporter,
    only_fixable: bool,
) -> i32 {
    let service = LintService::new(configuration, interner.clone(), source_manager);
    let result = service.run().await.unwrap_or_else(bail);
    let errored = result.has_errors();

    if only_fixable {
        reporter.report_all(result.only_fixable()).await;
    } else {
        reporter.report_all(result.into_iter()).await;
    }

    if errored {
        1
    } else {
        0
    }
}
