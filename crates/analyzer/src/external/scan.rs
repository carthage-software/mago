use std::sync::Arc;
use std::time::Instant;

use mago_database::GlobSettings;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_database::file::FileType;
use mago_database::matcher::ExclusionMatcher;
use mago_extension::PayloadWriter;
use mago_extension::source::SourceSnapshot;
use mago_names::ResolvedNames;
use mago_syntax::cst::Program;

use crate::external::AnalyzerTransport;
use crate::external::Backend;
use crate::external::ExternalAnalyzerError;
use crate::external::error::protocol;
use crate::external::protocol;

const BOOTSTRAP_GROUP: u64 = 0x434F_4445_5343_414E;
const MESSAGE_OVERHEAD: usize = 12 + 1 + 1 + 4 + 4;

#[derive(Debug, Clone)]
struct HookMatcher {
    index: u16,
    paths: ExclusionMatcher<String>,
}

#[derive(Debug, Clone)]
struct BackendPlan {
    backend: u16,
    hooks: Box<[HookMatcher]>,
}

/// Compiled source-file selectors advertised by enabled external plugins.
#[derive(Debug, Clone)]
pub struct CodebaseScanPlan {
    backends: Arc<[BackendPlan]>,
}

impl CodebaseScanPlan {
    pub(super) fn compile<T>(backends: &[Backend<T>]) -> Result<Option<Self>, ExternalAnalyzerError> {
        let mut plans = Vec::new();
        let mut hook_count = 0usize;
        let mut target_count = 0usize;
        for (backend, registered) in backends.iter().enumerate() {
            let mut hooks = Vec::with_capacity(registered.registration.codebase_scan_hooks.len());
            for hook in &registered.registration.codebase_scan_hooks {
                hook_count += 1;
                target_count += hook.targets.len();
                let paths = ExclusionMatcher::compile(hook.targets.iter().cloned(), GlobSettings::default()).map_err(
                    |error| {
                        protocol(format!(
                            "codebase-scan hook {} contains an invalid source-file target: {error}",
                            hook.index
                        ))
                    },
                )?;
                hooks.push(HookMatcher { index: hook.index, paths });
            }

            if !hooks.is_empty() {
                let backend = u16::try_from(backend)
                    .map_err(|_| protocol("more than 65,536 external analyzer backends were configured"))?;
                plans.push(BackendPlan { backend, hooks: hooks.into_boxed_slice() });
            }
        }

        tracing::trace!(
            backends = plans.len(),
            hooks = hook_count,
            targets = target_count,
            "Compiled external codebase-scan selectors."
        );
        Ok((!plans.is_empty()).then(|| Self { backends: plans.into() }))
    }

    /// Captures one matching host file while its first parsed syntax tree is live.
    ///
    /// Returns `None` without walking or encoding the tree when no hook target
    /// matches the file's logical path.
    ///
    /// # Errors
    ///
    /// Returns an error when the syntax snapshot exceeds protocol limits.
    pub fn capture(
        &self,
        file: &Arc<File>,
        program: &Program<'_>,
        resolved_names: &ResolvedNames<'_>,
    ) -> Result<Option<CodebaseScanFile>, ExternalAnalyzerError> {
        if file.file_type != FileType::Host {
            return Ok(None);
        }

        let Ok(path) = std::str::from_utf8(&file.name) else {
            return Ok(None);
        };
        let mut routes = Vec::new();
        for backend in self.backends.iter() {
            let hooks = backend
                .hooks
                .iter()
                .filter_map(|hook| hook.paths.is_match(path).then_some(hook.index))
                .collect::<Vec<_>>();
            if !hooks.is_empty() {
                routes.push(CodebaseScanRoute { backend: backend.backend, hooks: hooks.into_boxed_slice() });
            }
        }

        if routes.is_empty() {
            return Ok(None);
        }

        let snapshot = SourceSnapshot::complete_with_literals(program, resolved_names)?;
        let mut writer = PayloadWriter::with_capacity(snapshot.encoded_len_with_literals());
        snapshot.write_to_with_literals(&mut writer)?;
        Ok(Some(CodebaseScanFile {
            file: Arc::clone(file),
            snapshot: writer.finish().into(),
            routes: routes.into_boxed_slice(),
        }))
    }
}

#[derive(Debug, Clone)]
struct CodebaseScanRoute {
    backend: u16,
    hooks: Box<[u16]>,
}

/// An owned selected-source snapshot that can outlive its parser arena.
#[derive(Debug, Clone)]
pub struct CodebaseScanFile {
    file: Arc<File>,
    snapshot: Arc<[u8]>,
    routes: Box<[CodebaseScanRoute]>,
}

impl CodebaseScanFile {
    #[inline]
    #[must_use]
    pub fn file_id(&self) -> FileId {
        self.file.id
    }

    fn route(&self, backend: u16) -> Option<&[u16]> {
        self.routes.iter().find(|route| route.backend == backend).map(|route| route.hooks.as_ref())
    }

    fn encoded_len(&self, hooks: &[u16]) -> usize {
        4usize
            .saturating_add(hooks.len().saturating_mul(2))
            .saturating_add(4)
            .saturating_add(self.file.name.len())
            .saturating_add(4)
            .saturating_add(self.file.contents.len())
            .saturating_add(self.snapshot.len())
    }
}

pub(super) fn dispatch<T>(
    backends: &[Backend<T>],
    mut files: Vec<CodebaseScanFile>,
) -> Result<(), ExternalAnalyzerError>
where
    T: AnalyzerTransport,
{
    files.sort_unstable_by(|left, right| left.file.name.cmp(&right.file.name));
    for (backend_index, backend) in backends.iter().enumerate() {
        if backend.registration.codebase_scan_hooks.is_empty() {
            continue;
        }

        let backend_index = u16::try_from(backend_index)
            .map_err(|_| protocol("more than 65,536 external analyzer backends were configured"))?;
        let selected =
            files.iter().filter_map(|file| file.route(backend_index).map(|hooks| (file, hooks))).collect::<Vec<_>>();
        let active_hooks = backend.registration.codebase_scan_hooks.iter().map(|hook| hook.index).collect::<Vec<_>>();
        let maximum = backend.transport.maximum_payload_size();
        let batches = encode_batches(&active_hooks, &selected, maximum)?;
        let request_bytes = batches.iter().map(Vec::len).sum::<usize>();
        let started_at = tracing::enabled!(tracing::Level::TRACE).then(Instant::now);
        tracing::trace!(
            backend = backend_index,
            files = selected.len(),
            batches = batches.len(),
            request_bytes,
            "Broadcasting filtered codebase-scan snapshots."
        );
        let responses = backend.transport.broadcast_sequence(BOOTSTRAP_GROUP, &batches)?;
        for batch in responses {
            for response in batch {
                protocol::decode_codebase_scan_response(&response)?;
            }
        }
        if let Some(started_at) = started_at {
            tracing::trace!(
                backend = backend_index,
                files = selected.len(),
                batches = batches.len(),
                request_bytes,
                elapsed = ?started_at.elapsed(),
                "Filtered codebase-scan broadcast completed."
            );
        }
    }

    Ok(())
}

fn encode_batches(
    active_hooks: &[u16],
    files: &[(&CodebaseScanFile, &[u16])],
    maximum: usize,
) -> Result<Vec<Vec<u8>>, ExternalAnalyzerError> {
    let message_overhead = MESSAGE_OVERHEAD.saturating_add(active_hooks.len().saturating_mul(2));
    let mut ranges = Vec::new();
    let mut start = 0;
    let mut length = message_overhead;
    for (index, (file, hooks)) in files.iter().enumerate() {
        let record = file.encoded_len(hooks);
        if message_overhead.saturating_add(record) > maximum {
            return Err(protocol(format!(
                "codebase-scan snapshot for `{}` requires {} bytes, exceeding the worker payload limit of {maximum}",
                mago_bytes::BytesDisplay(&file.file.name),
                message_overhead.saturating_add(record),
            )));
        }
        if index > start && length.saturating_add(record) > maximum {
            ranges.push(start..index);
            start = index;
            length = message_overhead;
        }
        length = length.saturating_add(record);
    }
    ranges.push(start..files.len());

    let range_count = ranges.len();
    ranges
        .into_iter()
        .enumerate()
        .map(|(index, range)| {
            let mut writer = protocol::message_writer_with_capacity(
                protocol::CODEBASE_SCAN_REQUEST,
                files[range.clone()]
                    .iter()
                    .fold(message_overhead, |size, (file, hooks)| size.saturating_add(file.encoded_len(hooks))),
            );
            writer.write_bool(index == 0);
            writer.write_bool(index + 1 == range_count);
            writer.write_length(active_hooks.len())?;
            for hook in active_hooks {
                writer.write_u16(*hook);
            }
            writer.write_length(range.len())?;
            for (file, hooks) in &files[range] {
                writer.write_length(hooks.len())?;
                for hook in *hooks {
                    writer.write_u16(*hook);
                }
                writer.write_bytes(&file.file.name)?;
                writer.write_bytes(&file.file.contents)?;
                writer.write_raw(&file.snapshot);
            }
            Ok(writer.finish())
        })
        .collect()
}
