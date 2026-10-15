//! Decode file paths without changing the URI that identifies a document.

// File path conversion follows ls-types 0.0.6's URI helpers.
// Copyright (c) 2025 Milo Moisson
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use percent_encoding::AsciiSet;
use tower_lsp_server::gen_lsp_types::Uri;

const FILE_PATH: AsciiSet =
    percent_encoding::NON_ALPHANUMERIC.remove(b'-').remove(b'.').remove(b'_').remove(b'~').remove(b'/');

/// Convert file paths without changing URI text or resolving path aliases.
pub(super) trait UriFilePath: Sized {
    /// Encode an absolute path, or resolve an existing relative path.
    fn from_file_path(path: impl AsRef<Path>) -> Option<Self>;
    /// Decode the path while keeping parent components and symlinks intact.
    fn to_file_path(&self) -> Option<Cow<'_, Path>>;
}

impl UriFilePath for Uri {
    fn from_file_path(path: impl AsRef<Path>) -> Option<Self> {
        let path = path.as_ref();
        let path = if path.is_absolute() { Cow::Borrowed(path) } else { Cow::Owned(canonical_path(path)?) };
        #[cfg(windows)]
        let path = {
            let mut text = path.to_string_lossy().replace('\\', "/");
            if text.as_bytes().get(1) == Some(&b':') {
                text[..1].make_ascii_uppercase();
            }
            Cow::Owned(text)
        };
        #[cfg(not(windows))]
        let path = path.to_string_lossy();
        let prefix = if cfg!(windows) { "file:///" } else { "file://" };
        format!("{prefix}{}", percent_encoding::utf8_percent_encode(&path, &FILE_PATH)).parse().ok()
    }

    fn to_file_path(&self) -> Option<Cow<'_, Path>> {
        let decoded = self.path().decode().to_string_lossy();
        if decoded.is_empty() {
            return None;
        }
        let path = match decoded {
            Cow::Borrowed(path) => Cow::Borrowed(Path::new(path)),
            Cow::Owned(path) => Cow::Owned(PathBuf::from(path)),
        };
        #[cfg(windows)]
        {
            let host = self.authority().map(|authority| authority.host()).unwrap_or_default();
            if host.is_empty() {
                return Some(Cow::Owned(PathBuf::from(path.to_string_lossy().get(1..)?)));
            }
            return Some(Cow::Owned(Path::new(&format!("{host}:")).components().chain(path.components()).collect()));
        }
        #[cfg(not(windows))]
        Some(path)
    }
}

fn canonical_path(path: &Path) -> Option<PathBuf> {
    let path = path.canonicalize().ok()?;
    #[cfg(windows)]
    if let Some(std::path::Component::Prefix(prefix)) = path.components().next()
        && let std::path::Prefix::VerbatimDisk(drive) = prefix.kind()
    {
        return Some(
            Path::new(&format!("{}:", char::from(drive))).components().chain(path.components().skip(1)).collect(),
        );
    }
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn raw_aliases_and_parent_components_survive_path_conversion() {
        for raw in ["file:///tmp/link/../file.php", "file:///tmp/%66ile.php", "file:///tmp/file.php"] {
            let uri: Uri = raw.parse().unwrap();
            assert_eq!(uri.as_str(), raw);
            let expected = if raw.contains("link") { "/tmp/link/../file.php" } else { "/tmp/file.php" };
            assert_eq!(uri.to_file_path().as_deref(), Some(Path::new(expected)));
        }
        let first: Uri = "file:///tmp/%66ile.php".parse().unwrap();
        let second: Uri = "file:///tmp/file.php".parse().unwrap();
        assert_ne!(first, second);
    }

    #[test]
    fn file_paths_keep_existing_percent_encoding() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("café #['x'].php");
        let uri = Uri::from_file_path(&path).unwrap();
        assert!(uri.as_str().ends_with("caf%C3%A9%20%23%5B%27x%27%5D.php"));
        assert_eq!(uri.to_file_path().as_deref(), Some(path.as_path()));
    }
}
