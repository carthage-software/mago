//! `get_completion`: context-sensitive completion candidates.
//!
//! Completion contexts:
//! - **Variable** (`$<prefix>`): variables in the enclosing function scope.
//! - **Instance member** (`$obj-><prefix>`): receiver type comes from analyzer
//!   artifacts; methods, properties, and constants of the hierarchy are offered.
//! - **Static member** (`Class::<prefix>`): constants, enum cases, methods.
//! - **Bare identifier**: global functions, constants, class-likes (ranked by
//!   [`rank`]).
//! - **Qualified** (`Foo\<prefix>`, `\<prefix>`): namespace-scoped symbol search.
//! - **Import target**: global names of the imported kind, without call syntax.

mod context;
mod edit;
mod imports;
mod items;
mod locals;
mod members;
mod named_arguments;
mod native_type_context;
mod native_type_rules;
mod native_types;
mod rank;
mod site;
mod snippets;
mod symbols;

use mago_database::DatabaseReader;
use mago_database::file::FileId;

use crate::Server;
use crate::domain::{CompletionEntry, CompletionList};

use self::site::CompletionSite;

const MAX_RESULTS: usize = 50;

impl Server {
    /// Completion candidates for the cursor at `offset` in `file_id`.
    /// A typed space offers only declaration templates unless a name keyword precedes it.
    pub fn get_completion(&mut self, file_id: FileId, offset: u32, after_space: bool) -> CompletionList {
        let Ok(file) = self.database().get(&file_id) else { return CompletionList::default() };
        let tokens = self.lex(&file);
        if !site::allows_completion(&tokens, offset) {
            return CompletionList::default();
        }
        if after_space && !site::expects_name(&tokens, offset, self.php_version) {
            return CompletionList {
                is_incomplete: true,
                items: snippets::complete(&tokens, offset, self.php_version),
            };
        }

        let mut named = named_arguments::complete(self, file_id, &tokens, offset);
        if named.labels_only {
            items::document(self, &mut named.items);
            return CompletionList { is_incomplete: true, items: named.items };
        }
        let Some(analysis) = self.file_analysis_for(file_id) else { return CompletionList::default() };
        let Some(site) = CompletionSite::resolve(self, &file, &tokens, &analysis, offset) else {
            return CompletionList::default();
        };

        let mut items = site.candidates(self);
        site.apply_edit(&mut items);
        let mut preferred = site.native_types(self);
        preferred.extend(named.items);
        preferred.extend(snippets::complete(&tokens, offset, self.php_version));
        let mut items = prioritize(items, preferred);
        items::document(self, &mut items);
        CompletionList { is_incomplete: true, items }
    }
}

fn prioritize(items: Vec<CompletionEntry>, mut preferred: Vec<CompletionEntry>) -> Vec<CompletionEntry> {
    if preferred.is_empty() {
        return items;
    }
    preferred.extend(items);
    preferred.truncate(MAX_RESULTS);
    if !preferred[0].preselect || preferred[1..].iter().any(|entry| entry.preselect) {
        for entry in &mut preferred {
            entry.preselect = false;
        }
    }
    preferred
}
