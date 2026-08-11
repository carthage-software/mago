use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

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
    let trace_start = tracing::enabled!(tracing::Level::TRACE).then(Instant::now);
    tracing::trace!(
        configured_hosts = extension_hosts.len(),
        enabled_hosts = extension_hosts.values().filter(|host| host.enabled).count(),
        mago_threads,
        php_version = %php_version,
        "Initializing external linter hosts."
    );

    let pools = extension_hosts
        .iter()
        .filter(|(_, host)| host.enabled)
        .map(|(name, host)| {
            let host_start = tracing::enabled!(tracing::Level::TRACE).then(Instant::now);
            let command = host.worker_command().ok_or_else(|| {
                ExternalLintError::Protocol(format!("enabled extension host `{name}` has no command"))
            })?;

            let size = host.worker_count(mago_threads);
            let options = host.worker_pool_options();
            tracing::trace!(
                host = %name,
                command = ?command,
                workers = size.get(),
                adaptive = host.workers == 0,
                "Starting external linter host."
            );

            let pool = if host.workers == 0 {
                WorkerPool::spawn_adaptive(command, size, options)
            } else {
                WorkerPool::spawn(command, size, options)
            };

            let pool = pool.map(Arc::new).map_err(ExternalLintError::from)?;
            if let Some(start) = host_start {
                tracing::trace!(host = %name, active_workers = pool.len(), elapsed = ?start.elapsed(), "External linter host started.");
            }

            Ok::<Arc<WorkerPool>, ExternalLintError>(pool)
        })
        .collect::<Result<Vec<_>, _>>()?;

    if pools.is_empty() {
        tracing::trace!("No external linter hosts are enabled.");
        return Ok(None);
    }

    let linter = ExternalLinter::initialize(pools, php_version)?;
    if let Some(start) = trace_start {
        tracing::trace!(
            extensions = linter.extensions().len(),
            rules = linter.rules().len(),
            elapsed = ?start.elapsed(),
            "External linter hosts initialized."
        );
    }

    Ok(Some(linter))
}
