//! Lexical scopes and local bindings retained while walking one file.

use foldhash::HashMap;
use foldhash::HashSet;

use super::BindingKind;
use super::SymbolKey;
use super::SymbolKind;
use crate::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    File,
    Function,
    Method,
    Closure,
    ArrowFunction,
    PropertyHook,
}

#[derive(Debug, Clone, Copy)]
pub struct Scope {
    pub range: Range,
    pub parent: Option<u32>,
    pub kind: ScopeKind,
    pub is_static: bool,
}

struct Frame<'arena> {
    scope: u32,
    bindings: HashMap<&'arena [u8], u32>,
}

#[derive(Default)]
pub(super) struct Bindings<'arena> {
    pub symbols: Vec<SymbolKey>,
    pub scopes: Vec<Scope>,
    frames: Vec<Frame<'arena>>,
    writes: HashSet<u32>,
    declared: HashSet<u32>,
}

impl<'arena> Bindings<'arena> {
    pub fn enter(&mut self, range: Range, kind: ScopeKind, is_static: bool) {
        let scope = self.scopes.len() as u32;
        self.scopes.push(Scope { range, kind, parent: self.frames.last().map(|frame| frame.scope), is_static });
        self.frames.push(Frame { scope, bindings: HashMap::default() });
    }

    pub fn leave(&mut self) {
        self.frames.pop();
    }

    pub fn scope(&self) -> u32 {
        self.frames.last().map_or(0, |frame| frame.scope)
    }

    pub fn parameter(&mut self, name: &'arena [u8], offset: u32, promoted: bool) {
        let symbol = self.binding(self.frames.len() - 1, name, true);
        if promoted {
            self.symbols[symbol as usize].binding = Some(BindingKind::PromotedProperty);
        }
        self.write(offset);
    }

    pub fn implicit_parameter(&mut self, name: &'arena [u8]) {
        let symbol = self.binding(self.frames.len() - 1, name, true);
        self.declared.insert(symbol);
    }

    pub fn capture(&mut self, name: &'arena [u8]) {
        let parent = self.binding(self.frames.len() - 2, name, false);
        let current = self.frames.len() - 1;
        self.frames[current].bindings.insert(name, parent);
    }

    pub fn global(&mut self, name: &'arena [u8]) {
        let global = self.binding(0, name, false);
        let current = self.frames.len() - 1;
        self.frames[current].bindings.insert(name, global);
    }

    pub fn write(&mut self, offset: u32) {
        self.writes.insert(offset);
    }

    pub fn occurrence(&mut self, name: &'arena [u8], offset: u32) -> (u32, bool) {
        let symbol = self.binding(self.frames.len() - 1, name, false);
        let declaration = self.writes.contains(&offset) && self.declared.insert(symbol);
        (symbol, declaration)
    }

    fn binding(&mut self, frame: usize, name: &'arena [u8], parameter: bool) -> u32 {
        if let Some(id) = self.frames[frame].bindings.get(name) {
            return *id;
        }
        let scope = self.frames[frame].scope;
        let id = if !parameter && self.scopes[scope as usize].kind == ScopeKind::ArrowFunction {
            self.binding(frame - 1, name, false)
        } else {
            let id = self.symbols.len() as u32;
            self.symbols.push(SymbolKey {
                kind: SymbolKind::Variable,
                name: name.into(),
                scope: Some(scope),
                binding: Some(if parameter { BindingKind::Parameter } else { BindingKind::Local }),
            });
            id
        };
        self.frames[frame].bindings.insert(name, id);
        id
    }
}
