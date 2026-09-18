//! `get_folding_ranges`: the foldable regions of a file.
//!
//! These are precomputed during the per-file analysis pass, so this just hands
//! back the cached [`FoldRange`]s (byte offsets; the protocol layer maps them
//! to lines).

use mago_database::file::FileId;

use crate::Server;
use crate::domain::FoldRange;

impl Server {
    /// The foldable regions of `file_id`, as byte spans.
    pub fn get_folding_ranges(&mut self, file_id: FileId) -> Vec<FoldRange> {
        self.file_analysis_for(file_id).map(|analysis| analysis.fold_ranges.clone()).unwrap_or_default()
    }
}
