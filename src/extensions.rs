use std::collections::BTreeMap;
use std::sync::Arc;

use mago_extension::WorkerPool;
use mago_linter::external::ExternalLintError;
use mago_linter::external::ExternalLinter;
use mago_php_version::PHPVersion;

use crate::config::extension::ExtensionHostConfiguration;

/// Starts every enabled extension host and validates its linter registration.
pub(crate) fn initialize_external_linter(
    extension_hosts: &BTreeMap<String, ExtensionHostConfiguration>,
    php_version: PHPVersion,
    mago_threads: usize,
) -> Result<Option<ExternalLinter>, ExternalLintError> {
    let pools = extension_hosts
        .iter()
        .filter(|(_, host)| host.enabled)
        .map(|(name, host)| {
            let command = host.worker_command().ok_or_else(|| {
                ExternalLintError::Protocol(format!("enabled extension host `{name}` has no command"))
            })?;
            let size = host.worker_count(mago_threads);
            let options = host.worker_pool_options();
            let pool = if host.workers == 0 {
                WorkerPool::spawn_adaptive(command, size, options)
            } else {
                WorkerPool::spawn(command, size, options)
            };
            pool.map(Arc::new).map_err(ExternalLintError::from)
        })
        .collect::<Result<Vec<_>, _>>()?;

    if pools.is_empty() {
        return Ok(None);
    }

    ExternalLinter::initialize(pools, php_version).map(Some)
}
