use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_formatter_core::utils::column_after;

use crate::settings::FormatSettings;

const SPACES: [u8; 128] = [b' '; 128];
const TABS: [u8; 32] = [b'\t'; 32];

pub(super) struct Alignment<'arena> {
    settings: FormatSettings,
    cached_width: usize,
    cached_bytes: &'arena [u8],
}

impl<'arena> Alignment<'arena> {
    pub fn new(settings: FormatSettings) -> Self {
        Self { settings, cached_width: 0, cached_bytes: b"" }
    }

    pub fn get<A>(&mut self, arena: &'arena A, width: usize) -> &'arena [u8]
    where
        A: Arena,
    {
        let (tabs, spaces) = if self.settings.use_tabs && self.settings.tab_width != 0 {
            (width / self.settings.tab_width, width % self.settings.tab_width)
        } else {
            (0, width)
        };

        if tabs == 0 && spaces <= SPACES.len() {
            return &SPACES[..spaces];
        }
        if spaces == 0 && tabs <= TABS.len() {
            return &TABS[..tabs];
        }
        if self.cached_width == width {
            return self.cached_bytes;
        }

        let mut padding = Vec::with_capacity_in(tabs + spaces, arena);
        padding.resize(tabs, b'\t');
        padding.resize(tabs + spaces, b' ');
        self.cached_width = width;
        self.cached_bytes = padding.leak();
        self.cached_bytes
    }
}

pub(super) fn indentation_width(indentation: &[u8], tab_width: usize) -> usize {
    column_after(indentation, 0, tab_width)
}
