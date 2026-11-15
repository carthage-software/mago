//! Keep initial diagnostics until publication, then retain only fingerprints.

use std::io;

use foldhash::HashMap;
use tower_lsp_server::gen_lsp_types::Diagnostic;
use tower_lsp_server::gen_lsp_types::Uri;
use xxhash_rust::xxh3::Xxh3;

pub struct PublishedDiagnostics {
    pub diagnostics: Vec<Diagnostic>,
    pub version: Option<i32>,
}

impl PublishedDiagnostics {
    fn is_clear(&self) -> bool {
        self.diagnostics.is_empty() && self.version.is_none()
    }

    fn fingerprint(&self) -> Option<Fingerprint> {
        let mut hash = DiagnosticHasher(Xxh3::new());
        serde_json::to_writer(&mut hash, &self.diagnostics).ok()?;
        Some(Fingerprint { content: hash.0.digest128(), version: self.version })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Fingerprint {
    content: u128,
    version: Option<i32>,
}

struct DiagnosticHasher(Xxh3);

impl io::Write for DiagnosticHasher {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct DiagnosticPublication {
    pending: Option<HashMap<Uri, PublishedDiagnostics>>,
    published: HashMap<Uri, Option<Fingerprint>>,
}

impl DiagnosticPublication {
    pub fn new(initial: HashMap<Uri, PublishedDiagnostics>) -> Self {
        Self { pending: Some(initial), published: HashMap::default() }
    }

    /// Moves initial payloads to the publisher once. Later calls return nothing.
    pub fn take_initial(&mut self) -> Vec<(Uri, PublishedDiagnostics)> {
        let Some(initial) = self.pending.take() else {
            return Vec::new();
        };
        self.update(initial)
    }

    /// Queues changes before initial publication; afterward, returns changed payloads.
    pub fn update(&mut self, diagnostics: HashMap<Uri, PublishedDiagnostics>) -> Vec<(Uri, PublishedDiagnostics)> {
        if let Some(pending) = &mut self.pending {
            let previous_capacity = pending.capacity();
            for (uri, payload) in diagnostics {
                if payload.is_clear() {
                    pending.remove(&uri);
                } else {
                    pending.insert(uri, payload);
                }
            }
            if previous_capacity.max(pending.capacity()) > pending.len().saturating_mul(4).max(64) {
                pending.shrink_to_fit();
            }
            return Vec::new();
        }

        let previous_capacity = self.published.capacity();
        let changes = diagnostics
            .into_iter()
            .filter_map(|(uri, payload)| {
                if payload.is_clear() {
                    return self.published.remove(&uri).map(|_| (uri, payload));
                }
                let fingerprint = payload.fingerprint();
                if fingerprint.is_some() && self.published.get(&uri) == Some(&fingerprint) {
                    return None;
                }
                // A failed hash must never suppress a diagnostic.
                self.published.insert(uri.clone(), fingerprint);
                Some((uri, payload))
            })
            .collect();
        if previous_capacity.max(self.published.capacity()) > self.published.len().saturating_mul(4).max(64) {
            self.published.shrink_to_fit();
        }
        changes
    }

    pub fn uris(&self) -> impl Iterator<Item = &Uri> {
        self.published.keys().chain(self.pending.iter().flat_map(HashMap::keys))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn uri() -> Uri {
        "file:///test.php".parse().unwrap()
    }

    fn payload(message: &str, version: Option<i32>) -> PublishedDiagnostics {
        PublishedDiagnostics {
            diagnostics: vec![Diagnostic { message: message.into(), ..Diagnostic::default() }],
            version,
        }
    }

    fn one(payload: PublishedDiagnostics) -> HashMap<Uri, PublishedDiagnostics> {
        HashMap::from_iter([(uri(), payload)])
    }

    #[test]
    fn bulk_clears_release_sparse_tables_before_and_after_initial_publication() {
        let uris: Vec<Uri> = (0..256).map(|index| format!("file:///file{index}.php").parse().unwrap()).collect();
        for before_initial in [false, true] {
            let initial = uris.iter().cloned().map(|uri| (uri, payload("issue", Some(7)))).collect();
            let mut publication = DiagnosticPublication::new(initial);
            if !before_initial {
                assert_eq!(publication.take_initial().len(), uris.len());
                assert!(publication.published.capacity() >= uris.len());
            }
            let clears = uris[1..]
                .iter()
                .cloned()
                .map(|uri| (uri, PublishedDiagnostics { diagnostics: Vec::new(), version: None }))
                .collect();
            let output = publication.update(clears);
            if before_initial {
                assert!(output.is_empty());
                let pending = publication.pending.as_ref().unwrap();
                assert_eq!(pending.len(), 1);
                assert!(pending.capacity() < 64);
                let initial = publication.take_initial();
                assert_eq!(initial.len(), 1);
                assert_eq!(initial[0].0, uris[0]);
                assert_eq!(initial[0].1.version, Some(7));
                assert_eq!(initial[0].1.diagnostics[0].message, "issue".into());
            } else {
                assert_eq!(output.len(), uris.len() - 1);
                assert!(output.iter().all(|(uri, payload)| *uri != uris[0] && payload.is_clear()));
            }
            assert_eq!(publication.published.len(), 1);
            assert!(publication.published.capacity() < 64);
            assert!(publication.update(HashMap::from_iter([(uris[0].clone(), payload("issue", Some(7)))])).is_empty());
        }
    }

    #[test]
    fn initial_payloads_move_once_and_release_the_wire_cache() {
        let initial = payload("initial", None);
        let buffer = initial.diagnostics.as_ptr();
        let mut publication = DiagnosticPublication::new(one(initial));
        assert!(publication.published.is_empty());
        let output = publication.take_initial();
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].1.diagnostics.as_ptr(), buffer);
        assert!(publication.pending.is_none());
        assert_eq!(publication.published.len(), 1);
        assert!(publication.take_initial().is_empty());
        assert!(publication.update(one(payload("initial", None))).is_empty());
    }

    #[test]
    fn pending_changes_keep_the_latest_version_and_clear_deleted_files() {
        let mut publication = DiagnosticPublication::new(one(payload("old", None)));
        assert!(publication.update(one(payload("new", Some(2)))).is_empty());
        let output = publication.take_initial();
        assert_eq!(output[0].1.diagnostics[0].message, "new".into());
        assert_eq!(output[0].1.version, Some(2));

        let mut removed = DiagnosticPublication::new(one(payload("old", None)));
        assert!(removed.update(one(PublishedDiagnostics { diagnostics: vec![], version: None })).is_empty());
        assert!(removed.take_initial().is_empty());
    }

    #[test]
    fn versions_unknown_fingerprints_and_clears_publish() {
        let mut publication = DiagnosticPublication::default();
        assert_eq!(publication.update(one(payload("issue", Some(1)))).len(), 1);
        assert!(publication.update(one(payload("issue", Some(1)))).is_empty());
        assert_eq!(publication.update(one(payload("issue", Some(2)))).len(), 1);
        publication.published.insert(uri(), None);
        assert_eq!(publication.update(one(payload("issue", Some(2)))).len(), 1);
        assert_eq!(publication.update(one(PublishedDiagnostics { diagnostics: vec![], version: Some(3) })).len(), 1);
        assert_eq!(publication.update(one(PublishedDiagnostics { diagnostics: vec![], version: None })).len(), 1);
        assert!(publication.update(one(PublishedDiagnostics { diagnostics: vec![], version: None })).is_empty());
        assert_eq!(publication.update(one(payload("restored", None))).len(), 1);
        assert_eq!(publication.uris().cloned().collect::<Vec<_>>(), vec![uri()]);
    }

    #[test]
    fn fingerprint_covers_every_wire_field() {
        let original = payload("issue", None);
        let before = original.fingerprint().unwrap();
        let range = json!({"start": {"line": 1, "character": 2}, "end": {"line": 1, "character": 3}});
        for (field, value) in [
            ("range", range.clone()),
            ("severity", json!(1)),
            ("code", json!("new-code")),
            ("codeDescription", json!({"href": "https://example.com/code"})),
            ("source", json!("new-source")),
            ("message", json!("issue\n\nnote: note\n\nhelp: help")),
            (
                "relatedInformation",
                json!([{"location": {"uri": "file:///other.php", "range": range}, "message": "related"}]),
            ),
            ("tags", json!([1])),
            ("data", json!({"nested": [1, "value"]})),
        ] {
            let mut diagnostic = serde_json::to_value(&original.diagnostics[0]).unwrap();
            diagnostic[field] = value;
            let changed =
                PublishedDiagnostics { diagnostics: vec![serde_json::from_value(diagnostic).unwrap()], version: None };
            assert_ne!(before, changed.fingerprint().unwrap(), "{field}");
        }
    }
}
