use std::collections::hash_map::Entry;
use std::hash::Hash;

use foldhash::HashMap;
use foldhash::HashSet;
use mago_word::empty_word;

use mago_word::Word;
use mago_word::WordSet;

use crate::context::ScopeContext;
use crate::diff::CodebaseDiff;
use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::identifier::method::MethodIdentifier;
use crate::symbol::SymbolIdentifier;

/// Represents the source of a reference, distinguishing between top-level symbols
/// and members within a class-like structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ReferenceSource {
    /// A reference from a top-level symbol (function, class, enum, trait, interface, constant).
    /// The bool indicates if the reference occurs within a signature context (true) or body (false).
    /// The Word is the name (FQCN or FQN) of the referencing symbol.
    Symbol(bool, Word),
    /// A reference from a member within a class-like structure (method, property, class constant, enum case).
    /// The bool indicates if the reference occurs within a signature context (true) or body (false).
    /// The first Word is the FQCN of the class-like structure.
    /// The second Word is the name of the member.
    ClassLikeMember(bool, Word, Word),
    /// A reference from top-level code in a source file.
    /// The bool indicates if the reference occurs within a signature context.
    File(bool, Word),
}

/// Identifies where a recorded symbol reference originates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReferenceOrigin {
    /// A top-level symbol or class-like member.
    Symbol(SymbolIdentifier),
    /// Top-level code in a source file.
    File(Word),
}

/// Describes the semantic role of a recorded symbol reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SymbolReferenceKind {
    /// A reference from executable code.
    Body,
    /// A reference from a declaration signature.
    Signature,
    /// A reference to an overridden member.
    OverriddenMember,
    /// A use of another function-like's return value.
    FunctionLikeReturn,
    /// A property value read.
    PropertyRead,
    /// A property write.
    PropertyWrite,
}

/// Stores various maps tracking references between symbols (classes, functions, etc.)
/// and class-like members (methods, properties, constants, etc.) within the codebase.
///
/// This is primarily used for dependency analysis, understanding code structure,
/// and potentially for tasks like dead code detection or impact analysis.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::struct_field_names)]
pub struct SymbolReferences {
    /// Maps a referencing symbol/member `(RefSymbol, RefMember)` to a set of referenced symbols/members `(Symbol, Member)`
    /// found within the *body* of the referencing context.
    /// `RefMember` or `Member` being empty usually signifies the symbol itself.
    symbol_references_to_symbols: HashMap<SymbolIdentifier, HashSet<SymbolIdentifier>>,

    /// Maps a referencing symbol/member `(RefSymbol, RefMember)` to a set of referenced symbols/members `(Symbol, Member)`
    /// found within the *signature* (e.g., type hints, attributes) of the referencing context.
    symbol_references_to_symbols_in_signature: HashMap<SymbolIdentifier, HashSet<SymbolIdentifier>>,

    /// Maps a referencing symbol/member `(RefSymbol, RefMember)` to a set of *overridden* members `(ParentSymbol, Member)`
    /// that it directly references (e.g., via `parent::method()`).
    symbol_references_to_overridden_members: HashMap<SymbolIdentifier, HashSet<SymbolIdentifier>>,

    /// Maps a referencing function/method (`FunctionLikeIdentifier`) to a set of functions/methods (`FunctionLikeIdentifier`)
    /// whose return values it references/uses. Used for dead code analysis on return values.
    functionlike_references_to_functionlike_returns: HashMap<FunctionLikeIdentifier, HashSet<FunctionLikeIdentifier>>,

    /// Maps a logical file name to a set of referenced symbols/members `(Symbol, Member)`
    /// found within the file's global scope (outside any symbol). This tracks references from top-level code.
    /// Used for incremental analysis to determine which files need re-analysis when a symbol changes.
    file_references_to_symbols: HashMap<Word, HashSet<SymbolIdentifier>>,

    /// Maps a logical file name to a set of referenced symbols/members `(Symbol, Member)`
    /// found within the file's global scope signatures (e.g., top-level type declarations).
    file_references_to_symbols_in_signature: HashMap<Word, HashSet<SymbolIdentifier>>,

    /// Maps a referencing symbol/member to a set of properties that are *written* (assigned to).
    /// This is separate from read references to enable detection of write-only properties.
    /// The key is the referencing symbol/member, the value is the set of properties being written.
    property_write_references: HashMap<SymbolIdentifier, HashSet<SymbolIdentifier>>,

    /// Maps a referencing symbol/member to a set of properties that are *read* (accessed for value).
    /// This is separate from write references to enable accurate read/write tracking.
    /// The key is the referencing symbol/member, the value is the set of properties being read.
    property_read_references: HashMap<SymbolIdentifier, HashSet<SymbolIdentifier>>,

    /// Maps a logical file name to properties written from top-level code or a file-scoped closure.
    file_property_write_references: HashMap<Word, HashSet<SymbolIdentifier>>,

    /// Maps a logical file name to properties read from top-level code or a file-scoped closure.
    file_property_read_references: HashMap<Word, HashSet<SymbolIdentifier>>,
}

impl SymbolReferences {
    /// Creates a new, empty `SymbolReferences` collection.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether no references of any kind are recorded.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.symbol_references_to_symbols.is_empty()
            && self.symbol_references_to_symbols_in_signature.is_empty()
            && self.symbol_references_to_overridden_members.is_empty()
            && self.functionlike_references_to_functionlike_returns.is_empty()
            && self.file_references_to_symbols.is_empty()
            && self.file_references_to_symbols_in_signature.is_empty()
            && self.property_write_references.is_empty()
            && self.property_read_references.is_empty()
            && self.file_property_write_references.is_empty()
            && self.file_property_read_references.is_empty()
    }

    /// Counts the total number of body references from symbols and files.
    #[inline]
    pub fn count_body_references(&self) -> usize {
        self.symbol_references_to_symbols.values().map(std::collections::HashSet::len).sum::<usize>()
            + self.file_references_to_symbols.values().map(std::collections::HashSet::len).sum::<usize>()
    }

    /// Counts the total number of signature references from symbols and files.
    #[inline]
    pub fn count_signature_references(&self) -> usize {
        self.symbol_references_to_symbols_in_signature.values().map(std::collections::HashSet::len).sum::<usize>()
            + self.file_references_to_symbols_in_signature.values().map(std::collections::HashSet::len).sum::<usize>()
    }

    /// Returns the total number of map entries (keys) across all reference maps.
    /// Useful for memory auditing — this count should remain stable across cycles
    /// in a long-running process.
    #[inline]
    #[must_use]
    pub fn total_map_entries(&self) -> usize {
        self.symbol_references_to_symbols.len()
            + self.symbol_references_to_symbols_in_signature.len()
            + self.symbol_references_to_overridden_members.len()
            + self.functionlike_references_to_functionlike_returns.len()
            + self.file_references_to_symbols.len()
            + self.file_references_to_symbols_in_signature.len()
            + self.property_write_references.len()
            + self.property_read_references.len()
            + self.file_property_write_references.len()
            + self.file_property_read_references.len()
    }

    /// Counts how many symbols reference the given symbol.
    ///
    /// # Arguments
    /// * `symbol` - The symbol to check references to
    /// * `in_signature` - If true, count signature references; if false, count body references
    ///
    /// # Returns
    /// The number of symbols that reference the given symbol
    #[inline]
    #[must_use]
    pub fn count_referencing_symbols(&self, symbol: &SymbolIdentifier, in_signature: bool) -> usize {
        let map = if in_signature {
            &self.symbol_references_to_symbols_in_signature
        } else {
            &self.symbol_references_to_symbols
        };

        let files =
            if in_signature { &self.file_references_to_symbols_in_signature } else { &self.file_references_to_symbols };

        map.values().filter(|referenced_set| referenced_set.contains(symbol)).count()
            + files.values().filter(|referenced_set| referenced_set.contains(symbol)).count()
    }

    /// Returns whether a symbol or file references the target in a body or signature.
    #[inline]
    #[must_use]
    pub fn has_reference_to_symbol(&self, target_symbol: SymbolIdentifier) -> bool {
        self.referencing_symbols(target_symbol).next().is_some() || self.has_file_reference_to_symbol(target_symbol)
    }

    /// Counts how many symbols have a *read* reference to the given property.
    ///
    /// # Arguments
    ///
    /// * `property` - The property symbol identifier `(ClassName, PropertyName)` to check
    ///
    /// # Returns
    ///
    /// The number of symbols that read the given property
    #[inline]
    #[must_use]
    pub fn count_property_reads(&self, property: &SymbolIdentifier) -> usize {
        self.property_read_references.values().filter(|read_set| read_set.contains(property)).count()
            + self.file_property_read_references.values().filter(|read_set| read_set.contains(property)).count()
    }

    /// Returns whether any symbol or file reads the property.
    #[inline]
    #[must_use]
    pub fn has_property_read(&self, property: &SymbolIdentifier) -> bool {
        self.property_read_references.values().any(|references| references.contains(property))
            || self.file_property_read_references.values().any(|references| references.contains(property))
    }

    /// Counts how many symbols have a *write* reference to the given property.
    ///
    /// # Arguments
    ///
    /// * `property` - The property symbol identifier `(ClassName, PropertyName)` to check
    ///
    /// # Returns
    ///
    /// The number of symbols that write to the given property
    #[inline]
    #[must_use]
    pub fn count_property_writes(&self, property: &SymbolIdentifier) -> usize {
        self.property_write_references.values().filter(|write_set| write_set.contains(property)).count()
            + self.file_property_write_references.values().filter(|write_set| write_set.contains(property)).count()
    }

    /// Returns whether any symbol or file writes the property.
    #[inline]
    #[must_use]
    pub fn has_property_write(&self, property: &SymbolIdentifier) -> bool {
        self.property_write_references.values().any(|references| references.contains(property))
            || self.file_property_write_references.values().any(|references| references.contains(property))
    }

    /// Records that a top-level symbol (e.g., a function) references a class member.
    ///
    /// Automatically adds a reference from the referencing symbol to the member's class.
    ///
    /// # Arguments
    ///
    /// * `referencing_symbol`: The FQN of the function or global const making the reference.
    /// * `class_member`: A tuple `(ClassName, MemberName)` being referenced.
    /// * `in_signature`: `true` if the reference occurs in a signature context, `false` if in the body.
    #[inline]
    pub fn add_symbol_reference_to_class_member(
        &mut self,
        referencing_symbol: Word,
        class_member: SymbolIdentifier,
        in_signature: bool,
    ) {
        // Reference the class itself implicitly (in body context)
        self.add_symbol_reference_to_symbol(referencing_symbol, class_member.0, false);

        // Use empty member for the referencing symbol key
        let key = (referencing_symbol, empty_word());
        insert_dependency(
            &mut self.symbol_references_to_symbols,
            &mut self.symbol_references_to_symbols_in_signature,
            key,
            class_member,
            in_signature,
        );
    }

    /// Records that a top-level symbol references another top-level symbol.
    ///
    /// Skips self-references. Skips body references if already referenced in signature.
    ///
    /// # Arguments
    /// * `referencing_symbol`: The FQN of the symbol making the reference.
    /// * `symbol`: The FQN of the symbol being referenced.
    /// * `in_signature`: `true` if the reference occurs in a signature context, `false` if in the body.
    #[inline]
    pub fn add_symbol_reference_to_symbol(&mut self, referencing_symbol: Word, symbol: Word, in_signature: bool) {
        if referencing_symbol == symbol {
            return;
        }

        // Represent top-level symbols with an empty member identifier
        let referencing_key = (referencing_symbol, empty_word());
        let referenced_key = (symbol, empty_word());

        insert_dependency(
            &mut self.symbol_references_to_symbols,
            &mut self.symbol_references_to_symbols_in_signature,
            referencing_key,
            referenced_key,
            in_signature,
        );
    }

    /// Records a reference between arbitrary top-level symbols or class-like members.
    #[inline]
    pub fn add_symbol_reference(
        &mut self,
        referencing: SymbolIdentifier,
        referenced: SymbolIdentifier,
        in_signature: bool,
    ) {
        match (referencing.1.is_empty(), referenced.1.is_empty()) {
            (true, true) => self.add_symbol_reference_to_symbol(referencing.0, referenced.0, in_signature),
            (true, false) => self.add_symbol_reference_to_class_member(referencing.0, referenced, in_signature),
            (false, true) => self.add_class_member_reference_to_symbol(referencing, referenced.0, in_signature),
            (false, false) => self.add_class_member_reference_to_class_member(referencing, referenced, in_signature),
        }
    }

    /// Records a body or signature reference from a symbol/member or file origin.
    #[inline]
    pub fn add_reference(&mut self, referencing: ReferenceOrigin, referenced: SymbolIdentifier, in_signature: bool) {
        match referencing {
            ReferenceOrigin::Symbol(referencing) => {
                self.add_symbol_reference(referencing, referenced, in_signature);
            }
            ReferenceOrigin::File(file) => {
                self.add_file_reference_to_class_member(file, referenced, in_signature);
            }
        }
    }

    /// Records a property read from an explicit symbol, member, or file source.
    #[inline]
    pub fn add_property_read_reference(&mut self, referencing: ReferenceOrigin, property: SymbolIdentifier) {
        self.add_reference(referencing, property, false);
        match referencing {
            ReferenceOrigin::Symbol(symbol) => {
                insert_reference(&mut self.property_read_references, symbol, property);
            }
            ReferenceOrigin::File(file) => {
                insert_reference(&mut self.file_property_read_references, file, property);
            }
        }
    }

    /// Records a property write from an explicit symbol, member, or file source.
    #[inline]
    pub fn add_property_write_reference(&mut self, referencing: ReferenceOrigin, property: SymbolIdentifier) {
        self.add_reference(referencing, property, false);
        match referencing {
            ReferenceOrigin::Symbol(symbol) => {
                insert_reference(&mut self.property_write_references, symbol, property);
            }
            ReferenceOrigin::File(file) => {
                insert_reference(&mut self.file_property_write_references, file, property);
            }
        }
    }

    /// Records an explicit reference to an overridden class-like member.
    #[inline]
    pub fn add_overridden_member_reference(&mut self, referencing: SymbolIdentifier, overridden: SymbolIdentifier) {
        insert_reference(&mut self.symbol_references_to_overridden_members, referencing, overridden);
    }

    /// Records an explicit use of another function-like's return value.
    #[inline]
    pub fn add_functionlike_return_reference(&mut self, referencing: SymbolIdentifier, referenced: SymbolIdentifier) {
        let referencing = if referencing.1.is_empty() {
            FunctionLikeIdentifier::Function(referencing.0)
        } else {
            FunctionLikeIdentifier::Method(referencing.0, referencing.1)
        };
        let referenced = if referenced.1.is_empty() {
            FunctionLikeIdentifier::Function(referenced.0)
        } else {
            FunctionLikeIdentifier::Method(referenced.0, referenced.1)
        };

        self.add_reference_to_functionlike_return(referencing, referenced);
    }

    /// Records that a class member references another class member.
    ///
    /// Automatically adds references from the referencing member's class to the referenced member's class,
    /// and from the referencing member to the referenced member's class. Skips self-references.
    ///
    /// # Arguments
    /// * `referencing_class_member`: Tuple `(ClassName, MemberName)` making the reference.
    /// * `class_member`: Tuple `(ClassName, MemberName)` being referenced.
    /// * `in_signature`: `true` if the reference occurs in a signature context, `false` if in the body.
    #[inline]
    pub fn add_class_member_reference_to_class_member(
        &mut self,
        referencing_class_member: SymbolIdentifier,
        class_member: SymbolIdentifier,
        in_signature: bool,
    ) {
        if referencing_class_member == class_member {
            return;
        }

        // Add implicit references between the classes/symbols involved
        self.add_symbol_reference_to_symbol(referencing_class_member.0, class_member.0, false);
        self.add_class_member_reference_to_symbol(referencing_class_member, class_member.0, false);

        // Add the direct member-to-member reference
        insert_dependency(
            &mut self.symbol_references_to_symbols,
            &mut self.symbol_references_to_symbols_in_signature,
            referencing_class_member,
            class_member,
            in_signature,
        );
    }

    /// Records that a class member references a top-level symbol.
    ///
    /// Automatically adds a reference from the referencing member's class to the referenced symbol.
    /// Skips references to the member's own class. Skips body references if already referenced in signature.
    ///
    /// # Arguments
    /// * `referencing_class_member`: Tuple `(ClassName, MemberName)` making the reference.
    /// * `symbol`: The FQN of the symbol being referenced.
    /// * `in_signature`: `true` if the reference occurs in a signature context, `false` if in the body.
    #[inline]
    pub fn add_class_member_reference_to_symbol(
        &mut self,
        referencing_class_member: SymbolIdentifier,
        symbol: Word,
        in_signature: bool,
    ) {
        if referencing_class_member.0 == symbol {
            return;
        }

        // Add implicit reference from the class to the symbol
        self.add_symbol_reference_to_symbol(referencing_class_member.0, symbol, false);

        // Represent the referenced symbol with an empty member identifier
        let referenced_key = (symbol, empty_word());

        insert_dependency(
            &mut self.symbol_references_to_symbols,
            &mut self.symbol_references_to_symbols_in_signature,
            referencing_class_member,
            referenced_key,
            in_signature,
        );
    }

    /// Adds a file-level reference to a symbol or class-like member.
    ///
    /// Member references also imply a reference to their containing class-like symbol.
    #[inline]
    pub fn add_file_reference_to_class_member(
        &mut self,
        file_name: Word,
        class_member: SymbolIdentifier,
        in_signature: bool,
    ) {
        if !class_member.1.is_empty() {
            self.add_file_reference_to_class_member(file_name, (class_member.0, empty_word()), false);
        }

        insert_dependency(
            &mut self.file_references_to_symbols,
            &mut self.file_references_to_symbols_in_signature,
            file_name,
            class_member,
            in_signature,
        );
    }

    /// Convenience method to add a reference *from* the current function context *to* a class member.
    /// Delegates to appropriate `add_*` methods based on the function context.
    #[inline]
    pub fn add_reference_to_class_member(
        &mut self,
        scope: &ScopeContext<'_>,
        class_member: SymbolIdentifier,
        in_signature: bool,
    ) {
        self.add_reference(scope.get_reference_origin(), class_member, in_signature);
    }

    #[inline]
    pub fn add_reference_for_method_call(&mut self, scope: &ScopeContext<'_>, method: &MethodIdentifier) {
        self.add_reference_to_class_member(
            scope,
            (method.get_class_name().to_ascii_lowercase(), method.get_method_name()),
            false,
        );
    }

    /// Records a read reference to a property (e.g., `$this->prop` used as a value).
    #[inline]
    pub fn add_reference_for_property_read(&mut self, scope: &ScopeContext<'_>, class_name: Word, property_name: Word) {
        let normalized_class_name = class_name.to_ascii_lowercase();
        let class_member = (normalized_class_name, property_name);

        self.add_property_read_reference(scope.get_reference_origin(), class_member);
    }

    /// Records a write reference to a property (e.g., `$this->prop = value`).
    /// This is tracked separately from read references to enable write-only property detection.
    #[inline]
    pub fn add_reference_for_property_write(
        &mut self,
        scope: &ScopeContext<'_>,
        class_name: Word,
        property_name: Word,
    ) {
        let normalized_class_name = class_name.to_ascii_lowercase();
        let class_member = (normalized_class_name, property_name);

        self.add_property_write_reference(scope.get_reference_origin(), class_member);
    }

    /// Convenience method to add a reference *from* the current function context *to* an overridden class member (e.g., `parent::foo`).
    /// Delegates based on the function context.
    #[inline]
    pub fn add_reference_to_overridden_class_member(&mut self, scope: &ScopeContext, class_member: SymbolIdentifier) {
        let ReferenceOrigin::Symbol(referencing) = scope.get_reference_origin() else {
            return;
        };

        self.add_overridden_member_reference(referencing, class_member);
    }

    /// Convenience method to add a reference *from* the current function context *to* a top-level symbol.
    /// Delegates to appropriate `add_*` methods based on the function context.
    #[inline]
    pub fn add_reference_to_symbol(&mut self, scope: &ScopeContext, symbol: Word, in_signature: bool) {
        self.add_reference(scope.get_reference_origin(), (symbol, empty_word()), in_signature);
    }

    /// Records that one function/method references the return value of another. Used for dead code analysis.
    #[inline]
    pub fn add_reference_to_functionlike_return(
        &mut self,
        referencing_functionlike: FunctionLikeIdentifier,
        referenced_functionlike: FunctionLikeIdentifier,
    ) {
        if referencing_functionlike == referenced_functionlike {
            return;
        }

        insert_reference(
            &mut self.functionlike_references_to_functionlike_returns,
            referencing_functionlike,
            referenced_functionlike,
        );
    }

    /// Merges references from another `SymbolReferences` instance into this one.
    /// Signature edges suppress matching body edges; other reference roles stay distinct.
    #[inline]
    pub fn extend(&mut self, other: Self) {
        extend_dependencies(
            &mut self.symbol_references_to_symbols,
            &mut self.symbol_references_to_symbols_in_signature,
            other.symbol_references_to_symbols,
            other.symbol_references_to_symbols_in_signature,
        );
        extend_reference_map(
            &mut self.symbol_references_to_overridden_members,
            other.symbol_references_to_overridden_members,
        );
        extend_reference_map(
            &mut self.functionlike_references_to_functionlike_returns,
            other.functionlike_references_to_functionlike_returns,
        );
        extend_dependencies(
            &mut self.file_references_to_symbols,
            &mut self.file_references_to_symbols_in_signature,
            other.file_references_to_symbols,
            other.file_references_to_symbols_in_signature,
        );
        extend_reference_map(&mut self.property_write_references, other.property_write_references);
        extend_reference_map(&mut self.property_read_references, other.property_read_references);
        extend_reference_map(&mut self.file_property_write_references, other.file_property_write_references);
        extend_reference_map(&mut self.file_property_read_references, other.file_property_read_references);
    }

    /// Visits every recorded reference without materializing a copy of the graph.
    #[inline]
    pub fn for_each_reference(&self, mut visit: impl FnMut(ReferenceOrigin, SymbolIdentifier, SymbolReferenceKind)) {
        for (source, targets) in &self.symbol_references_to_symbols {
            for target in targets {
                visit(ReferenceOrigin::Symbol(*source), *target, SymbolReferenceKind::Body);
            }
        }
        for (source, targets) in &self.symbol_references_to_symbols_in_signature {
            for target in targets {
                visit(ReferenceOrigin::Symbol(*source), *target, SymbolReferenceKind::Signature);
            }
        }
        for (source, targets) in &self.symbol_references_to_overridden_members {
            for target in targets {
                visit(ReferenceOrigin::Symbol(*source), *target, SymbolReferenceKind::OverriddenMember);
            }
        }
        for (source, targets) in &self.functionlike_references_to_functionlike_returns {
            let Some(source) = function_like_symbol_identifier(source) else {
                continue;
            };
            for target in targets {
                if let Some(target) = function_like_symbol_identifier(target) {
                    visit(ReferenceOrigin::Symbol(source), target, SymbolReferenceKind::FunctionLikeReturn);
                }
            }
        }
        for (source, targets) in &self.file_references_to_symbols {
            for target in targets {
                visit(ReferenceOrigin::File(*source), *target, SymbolReferenceKind::Body);
            }
        }
        for (source, targets) in &self.file_references_to_symbols_in_signature {
            for target in targets {
                visit(ReferenceOrigin::File(*source), *target, SymbolReferenceKind::Signature);
            }
        }
        for (source, targets) in &self.property_read_references {
            for target in targets {
                visit(ReferenceOrigin::Symbol(*source), *target, SymbolReferenceKind::PropertyRead);
            }
        }
        for (source, targets) in &self.property_write_references {
            for target in targets {
                visit(ReferenceOrigin::Symbol(*source), *target, SymbolReferenceKind::PropertyWrite);
            }
        }
        for (source, targets) in &self.file_property_read_references {
            for target in targets {
                visit(ReferenceOrigin::File(*source), *target, SymbolReferenceKind::PropertyRead);
            }
        }
        for (source, targets) in &self.file_property_write_references {
            for target in targets {
                visit(ReferenceOrigin::File(*source), *target, SymbolReferenceKind::PropertyWrite);
            }
        }
    }

    /// Finds all symbols/members that reference a specific target symbol/member.
    /// Checks both body and signature references.
    ///
    /// # Arguments
    ///
    /// * `target_symbol`: The `(SymbolName, MemberName)` tuple being referenced.
    ///
    /// # Returns
    ///
    /// A `HashSet` containing `&(RefSymbol, RefMember)` tuples of all items referencing the target.
    #[inline]
    #[must_use]
    pub fn get_references_to_symbol(&self, target_symbol: SymbolIdentifier) -> HashSet<&SymbolIdentifier> {
        self.referencing_symbols(target_symbol).collect()
    }

    /// Iterates over symbols that reference a target in a body or signature without allocating a set.
    /// A source appears twice if both roles reference the target.
    #[inline]
    pub fn referencing_symbols(&self, target_symbol: SymbolIdentifier) -> impl Iterator<Item = &SymbolIdentifier> {
        self.symbol_references_to_symbols
            .iter()
            .chain(&self.symbol_references_to_symbols_in_signature)
            .filter_map(move |(source, targets)| targets.contains(&target_symbol).then_some(source))
    }

    /// Returns whether a body or signature reference to a symbol originates from top-level file code.
    #[inline]
    #[must_use]
    pub fn has_file_reference_to_symbol(&self, target_symbol: SymbolIdentifier) -> bool {
        self.file_references_to_symbols.values().any(|references| references.contains(&target_symbol))
            || self
                .file_references_to_symbols_in_signature
                .values()
                .any(|references| references.contains(&target_symbol))
    }

    /// Calculates sets of invalid symbols and members based on detected code changes (`CodebaseDiff`).
    /// Propagates invalidation through the dependency graph stored in signature references.
    /// Limits propagation expense to avoid excessive computation on large changes.
    ///
    /// # Arguments
    ///
    /// * `codebase_diff`: Information about added, deleted, or modified symbols/signatures.
    ///
    /// # Returns
    ///
    /// `Some((invalid_signatures, partially_invalid, invalid_files))` on success, where `invalid_signatures` contains
    /// all symbol/member pairs whose signature is invalid (including propagated ones), and `partially_invalid`
    /// contains symbols with at least one invalid member. `invalid_files` contains logical file names whose
    /// top-level code references a symbol with an invalid signature.
    /// Returns `None` if the propagation exceeds an expense limit (currently 5000 steps).
    #[inline]
    #[must_use]
    pub fn get_invalid_symbols(
        &self,
        codebase_diff: &CodebaseDiff,
    ) -> Option<(HashSet<SymbolIdentifier>, WordSet, WordSet)> {
        let mut invalid_signatures = HashSet::default();
        let mut partially_invalid_symbols = WordSet::default();

        let mut sig_reverse_index: HashMap<SymbolIdentifier, Vec<SymbolIdentifier>> = HashMap::default();
        for (referencing_item, referenced_items) in &self.symbol_references_to_symbols_in_signature {
            let containing_symbol = (referencing_item.0, empty_word());
            if codebase_diff.contains_changed_entry(&containing_symbol) {
                invalid_signatures.insert(*referencing_item);
                partially_invalid_symbols.insert(referencing_item.0);
            }

            for referenced in referenced_items {
                sig_reverse_index.entry(*referenced).or_default().push(*referencing_item);
            }
        }

        // Start with symbols directly added/deleted in the diff.
        let mut symbols_to_process = codebase_diff.get_changed().iter().copied().collect::<Vec<_>>();
        let mut processed_symbols = HashSet::default();
        let mut expense_counter = 0;

        const EXPENSE_LIMIT: usize = 5000;
        while let Some(invalidated_item) = symbols_to_process.pop() {
            if processed_symbols.contains(&invalidated_item) {
                continue;
            }

            expense_counter += 1;
            if expense_counter > EXPENSE_LIMIT {
                return None;
            }

            // Mark this item as invalid (signature) and processed
            invalid_signatures.insert(invalidated_item);
            processed_symbols.insert(invalidated_item);
            if !invalidated_item.1.is_empty() {
                // If it's a member, also mark its containing symbol for processing.
                partially_invalid_symbols.insert(invalidated_item.0);
                let containing_symbol = (invalidated_item.0, empty_word());
                if !processed_symbols.contains(&containing_symbol) {
                    symbols_to_process.push(containing_symbol);
                }
            }

            // Find all items that reference this now-invalid item *in their signature*
            if let Some(referencing_items) = sig_reverse_index.get(&invalidated_item) {
                for referencing_item in referencing_items {
                    if !processed_symbols.contains(referencing_item) {
                        symbols_to_process.push(*referencing_item);
                    }

                    invalid_signatures.insert(*referencing_item);
                    if !referencing_item.1.is_empty() {
                        partially_invalid_symbols.insert(referencing_item.0);
                    }
                }
            }
        }

        // An item's body is invalid if it references (anywhere, body or sig) an item with an invalid signature.
        // Check both body and signature reference maps in a single pass where possible.
        let mut invalid_bodies = HashSet::default();

        for (referencing_item, referenced_items) in &self.symbol_references_to_symbols {
            if referenced_items.iter().any(|r| invalid_signatures.contains(r)) {
                invalid_bodies.insert(*referencing_item);
                if !referencing_item.1.is_empty() {
                    partially_invalid_symbols.insert(referencing_item.0);
                }
            }
        }

        for (referencing_item, referenced_items) in &self.symbol_references_to_symbols_in_signature {
            if referenced_items.iter().any(|r| invalid_signatures.contains(r)) {
                invalid_bodies.insert(*referencing_item);
                if !referencing_item.1.is_empty() {
                    partially_invalid_symbols.insert(referencing_item.0);
                }
            }
        }

        let mut invalid_files = WordSet::default();
        for (file, referenced_items) in
            self.file_references_to_symbols.iter().chain(&self.file_references_to_symbols_in_signature)
        {
            if referenced_items.iter().any(|referenced| invalid_signatures.contains(referenced)) {
                invalid_files.insert(*file);
            }
        }

        let mut all_invalid_symbols = invalid_signatures;
        all_invalid_symbols.extend(invalid_bodies);
        Some((all_invalid_symbols, partially_invalid_symbols, invalid_files))
    }

    /// Extracts references originating from safe (skipped) symbols and merges them into this instance.
    ///
    /// When incremental analysis runs with `diff = true`, the analyzer skips safe symbols,
    /// which means their body references are not collected. This method copies those missing
    /// references from the previous run's reference graph.
    ///
    /// Only references from symbols that are in `safe_symbols` or `safe_symbol_members`
    /// (and not already present in this instance) are copied.
    ///
    /// # Arguments
    ///
    /// * `previous` - The previous run's complete symbol references
    /// * `safe_symbols` - Set of safe top-level symbol names
    /// * `safe_symbol_members` - Set of safe (symbol, member) pairs
    #[inline]
    pub fn restore_references_for_safe_symbols(
        &mut self,
        previous: &SymbolReferences,
        safe_symbols: &WordSet,
        safe_symbol_members: &HashSet<SymbolIdentifier>,
    ) {
        let is_safe = |key: &SymbolIdentifier| -> bool {
            if key.1.is_empty() { safe_symbols.contains(&key.0) } else { safe_symbol_members.contains(key) }
        };

        // Restore body references for safe symbols
        for (key, refs) in &previous.symbol_references_to_symbols {
            if is_safe(key) && !self.symbol_references_to_symbols.contains_key(key) {
                let mut refs = refs.clone();
                if let Some(signature) = self.symbol_references_to_symbols_in_signature.get(key) {
                    refs.retain(|target| !signature.contains(target));
                }
                if !refs.is_empty() {
                    self.symbol_references_to_symbols.insert(*key, refs);
                }
            }
        }

        // Restore overridden member references for safe symbols
        for (key, refs) in &previous.symbol_references_to_overridden_members {
            if is_safe(key) && !self.symbol_references_to_overridden_members.contains_key(key) {
                self.symbol_references_to_overridden_members.insert(*key, refs.clone());
            }
        }

        // Restore function-like return references for safe symbols
        for (key, refs) in &previous.functionlike_references_to_functionlike_returns {
            let sym_key = match key {
                FunctionLikeIdentifier::Function(name) => (*name, mago_word::empty_word()),
                FunctionLikeIdentifier::Method(class, method) => (*class, *method),
                _ => continue,
            };

            if is_safe(&sym_key) && !self.functionlike_references_to_functionlike_returns.contains_key(key) {
                self.functionlike_references_to_functionlike_returns.insert(*key, refs.clone());
            }
        }

        // Restore property write references for safe symbols
        for (key, refs) in &previous.property_write_references {
            if is_safe(key) && !self.property_write_references.contains_key(key) {
                self.property_write_references.insert(*key, refs.clone());
            }
        }

        // Restore property read references for safe symbols
        for (key, refs) in &previous.property_read_references {
            if is_safe(key) && !self.property_read_references.contains_key(key) {
                self.property_read_references.insert(*key, refs.clone());
            }
        }
    }

    /// Removes **body** references originating from the given symbols/members.
    ///
    /// Used by the body-only fast path: when only function/method bodies changed (no signature
    /// changes), we remove old body references and let the analyzer rebuild them fresh.
    /// Signature references are kept because signatures didn't change.
    ///
    /// Also removes function-like return references and property read/write references from
    /// the given symbols, as those originate from body code.
    ///
    /// File-level references keyed by the given file names are also removed.
    #[inline]
    pub fn remove_body_references_for_symbols(
        &mut self,
        symbols_and_members: &HashSet<SymbolIdentifier>,
        file_names: &[Word],
    ) {
        // Remove body (not signature) references
        for key in symbols_and_members {
            self.symbol_references_to_symbols.remove(key);
            self.symbol_references_to_overridden_members.remove(key);
            self.property_write_references.remove(key);
            self.property_read_references.remove(key);
        }

        // Remove function-like return references for matching keys
        self.functionlike_references_to_functionlike_returns.retain(|key, _| {
            let sym_key = match key {
                FunctionLikeIdentifier::Function(name) => (*name, mago_word::empty_word()),
                FunctionLikeIdentifier::Method(class, method) => (*class, *method),
                _ => return true,
            };

            !symbols_and_members.contains(&sym_key)
        });

        // Remove file-level body references (signature refs kept)
        for name in file_names {
            self.file_references_to_symbols.remove(name);
            self.file_property_write_references.remove(name);
            self.file_property_read_references.remove(name);
        }
    }

    /// Removes every reference originating from the given files.
    ///
    /// Used when files are fully reanalyzed after a signature change. Both body and signature
    /// references must be rebuilt because either may have changed.
    #[inline]
    pub fn remove_references_from_files(&mut self, file_names: &WordSet) {
        for name in file_names {
            self.file_references_to_symbols.remove(name);
            self.file_references_to_symbols_in_signature.remove(name);
            self.file_property_write_references.remove(name);
            self.file_property_read_references.remove(name);
        }
    }

    /// Removes references whose source files no longer exist in the current codebase.
    #[inline]
    pub fn retain_references_from_files(&mut self, file_names: &WordSet) {
        self.file_references_to_symbols.retain(|name, _| file_names.contains(name));
        self.file_references_to_symbols_in_signature.retain(|name, _| file_names.contains(name));
        self.file_property_write_references.retain(|name, _| file_names.contains(name));
        self.file_property_read_references.retain(|name, _| file_names.contains(name));
    }

    /// Removes all references *originating from* symbols/members that are marked as invalid.
    ///
    /// # Arguments
    ///
    /// * `invalid_symbols_and_members`: A set containing `(SymbolName, MemberName)` tuples for invalid items.
    #[inline]
    pub fn remove_references_from_invalid_symbols(&mut self, invalid_symbols_and_members: &HashSet<SymbolIdentifier>) {
        // Retain only entries where the key (referencing item) is NOT in the invalid set.
        self.symbol_references_to_symbols
            .retain(|referencing_item, _| !invalid_symbols_and_members.contains(referencing_item));
        self.symbol_references_to_symbols_in_signature
            .retain(|referencing_item, _| !invalid_symbols_and_members.contains(referencing_item));
        self.symbol_references_to_overridden_members
            .retain(|referencing_item, _| !invalid_symbols_and_members.contains(referencing_item));
        self.property_write_references
            .retain(|referencing_item, _| !invalid_symbols_and_members.contains(referencing_item));
        self.property_read_references
            .retain(|referencing_item, _| !invalid_symbols_and_members.contains(referencing_item));
    }

    /// Retains only references originating from safe (unchanged) symbols, removing all others.
    ///
    /// This is the inverse of [`remove_references_from_invalid_symbols`]: instead of
    /// specifying what to remove, you specify what to keep. References from non-safe symbols
    /// will be rebuilt by `populate_codebase` and the analyzer.
    ///
    /// This method also retains all builtin/prelude references (those where the key symbol
    /// is not user-defined, i.e., is in the base references).
    #[inline]
    pub fn retain_safe_symbol_references(
        &mut self,
        safe_symbols: &WordSet,
        safe_symbol_members: &HashSet<SymbolIdentifier>,
    ) {
        let is_safe = |key: &SymbolIdentifier| -> bool {
            if key.1.is_empty() { safe_symbols.contains(&key.0) } else { safe_symbol_members.contains(key) }
        };

        self.symbol_references_to_symbols.retain(|k, _| is_safe(k));
        self.symbol_references_to_symbols_in_signature.retain(|k, _| is_safe(k));
        self.symbol_references_to_overridden_members.retain(|k, _| is_safe(k));
        self.property_write_references.retain(|k, _| is_safe(k));
        self.property_read_references.retain(|k, _| is_safe(k));

        self.functionlike_references_to_functionlike_returns.retain(|key, _| {
            let sym_key = match key {
                FunctionLikeIdentifier::Function(name) => (*name, mago_word::empty_word()),
                FunctionLikeIdentifier::Method(class, method) => (*class, *method),
                _ => return true, // Keep closures and other non-symbol function-likes
            };

            is_safe(&sym_key)
        });
    }

    /// Removes references for dirty (non-safe) symbols — O(dirty) instead of O(all).
    ///
    /// This is the inverse of [`retain_safe_symbol_references`]: instead of iterating all
    /// entries and keeping safe ones, it directly removes entries for the given dirty set.
    /// Much faster when the dirty set is small relative to the total number of references.
    pub fn remove_dirty_symbol_references(&mut self, dirty_symbols: &HashSet<SymbolIdentifier>) {
        for key in dirty_symbols {
            self.symbol_references_to_symbols.remove(key);
            self.symbol_references_to_symbols_in_signature.remove(key);
            self.symbol_references_to_overridden_members.remove(key);
            self.property_write_references.remove(key);
            self.property_read_references.remove(key);

            let fl_key = if key.1.is_empty() {
                FunctionLikeIdentifier::Function(key.0)
            } else {
                FunctionLikeIdentifier::Method(key.0, key.1)
            };

            self.functionlike_references_to_functionlike_returns.remove(&fl_key);
        }
    }
}

#[inline]
fn insert_dependency<K>(
    body: &mut HashMap<K, HashSet<SymbolIdentifier>>,
    signature: &mut HashMap<K, HashSet<SymbolIdentifier>>,
    origin: K,
    target: SymbolIdentifier,
    in_signature: bool,
) where
    K: Copy + Eq + Hash,
{
    if in_signature {
        insert_reference(signature, origin, target);
        if let Some(targets) = body.get_mut(&origin) {
            targets.remove(&target);
            if targets.is_empty() {
                body.remove(&origin);
            }
        }
    } else if signature.get(&origin).is_none_or(|targets| !targets.contains(&target)) {
        insert_reference(body, origin, target);
    }
}

fn extend_dependencies<K>(
    body: &mut HashMap<K, HashSet<SymbolIdentifier>>,
    signature: &mut HashMap<K, HashSet<SymbolIdentifier>>,
    mut incoming_body: HashMap<K, HashSet<SymbolIdentifier>>,
    incoming_signature: HashMap<K, HashSet<SymbolIdentifier>>,
) where
    K: Copy + Eq + Hash,
{
    for (origin, targets) in &incoming_signature {
        if let Some(existing) = body.get_mut(origin) {
            for target in targets {
                existing.remove(target);
            }

            if existing.is_empty() {
                body.remove(origin);
            }
        }
    }

    extend_reference_map(signature, incoming_signature);
    incoming_body.retain(|origin, targets| {
        if let Some(signature) = signature.get(origin) {
            targets.retain(|target| !signature.contains(target));
        }

        !targets.is_empty()
    });

    extend_reference_map(body, incoming_body);
}

#[inline]
fn insert_reference<K, V>(references: &mut HashMap<K, HashSet<V>>, origin: K, target: V)
where
    K: Eq + Hash,
    V: Eq + Hash,
{
    let targets = references.entry(origin).or_default();
    if targets.len() < targets.capacity() || !targets.contains(&target) {
        targets.insert(target);
    }
}

#[allow(clippy::set_contains_or_insert, reason = "A duplicate insert can grow a full set.")]
fn extend_reference_map<K, V>(target: &mut HashMap<K, HashSet<V>>, source: HashMap<K, HashSet<V>>)
where
    K: Eq + Hash,
    V: Eq + Hash,
{
    if source.is_empty() {
        return;
    }

    if target.is_empty() {
        *target = source;
        return;
    }

    for (key, references) in source {
        match target.entry(key) {
            Entry::Occupied(mut entry) => {
                let existing = entry.get_mut();
                for reference in references {
                    if existing.len() < existing.capacity() || !existing.contains(&reference) {
                        existing.insert(reference);
                    }
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(references);
            }
        }
    }
}

fn function_like_symbol_identifier(identifier: &FunctionLikeIdentifier) -> Option<SymbolIdentifier> {
    match identifier {
        FunctionLikeIdentifier::Function(name) => Some((*name, empty_word())),
        FunctionLikeIdentifier::Method(class, method) => Some((*class, *method)),
        FunctionLikeIdentifier::Closure(_) => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use mago_word::empty_word;
    use mago_word::word;

    fn make_refs_with_body(entries: Vec<(SymbolIdentifier, Vec<SymbolIdentifier>)>) -> SymbolReferences {
        let mut refs = SymbolReferences::new();
        for (key, values) in entries {
            let set: HashSet<SymbolIdentifier> = values.into_iter().collect();
            refs.symbol_references_to_symbols.insert(key, set);
        }
        refs
    }

    fn add_every_reference_kind(references: &mut SymbolReferences, source: Word, target: Word) {
        let symbol = (source, empty_word());
        let target = (target, empty_word());
        references.add_symbol_reference(symbol, target, false);
        references.add_symbol_reference(symbol, target, true);
        references.add_overridden_member_reference(symbol, target);
        references.add_functionlike_return_reference(symbol, target);
        references.add_property_read_reference(ReferenceOrigin::Symbol(symbol), target);
        references.add_property_write_reference(ReferenceOrigin::Symbol(symbol), target);
        references.add_reference(ReferenceOrigin::File(source), target, false);
        references.add_reference(ReferenceOrigin::File(source), target, true);
        references.add_property_read_reference(ReferenceOrigin::File(source), target);
        references.add_property_write_reference(ReferenceOrigin::File(source), target);
    }

    #[test]
    fn test_extend_preserves_all_reference_kinds_for_vacant_keys() {
        let mut incoming = SymbolReferences::new();
        add_every_reference_kind(&mut incoming, word("source"), word("target"));
        let expected = incoming.clone();
        let mut references = SymbolReferences::new();

        references.extend(incoming);

        assert_eq!(references, expected);
    }

    #[test]
    fn test_extend_unions_overlapping_keys_and_preserves_other_keys() {
        let mut references = SymbolReferences::new();
        add_every_reference_kind(&mut references, word("shared"), word("old_target"));
        add_every_reference_kind(&mut references, word("existing"), word("old_target"));

        let mut incoming = SymbolReferences::new();
        add_every_reference_kind(&mut incoming, word("shared"), word("old_target"));
        add_every_reference_kind(&mut incoming, word("shared"), word("new_target"));
        add_every_reference_kind(&mut incoming, word("new"), word("new_target"));

        let mut expected = references.clone();
        add_every_reference_kind(&mut expected, word("shared"), word("new_target"));
        add_every_reference_kind(&mut expected, word("new"), word("new_target"));
        references.extend(incoming);

        assert_eq!(references, expected);
    }

    #[test]
    fn merging_duplicate_references_keeps_set_capacity() {
        let mut targets = HashSet::default();
        targets.reserve(3);
        let capacity = targets.capacity();
        targets.extend(0..capacity);
        let mut references = HashMap::default();
        references.insert(0, targets);

        for _ in 0..3 {
            let mut incoming = HashMap::default();
            incoming.insert(0, (0..capacity).collect());
            extend_reference_map(&mut references, incoming);
            assert_eq!(references[&0].len(), capacity);
            assert_eq!(references[&0].capacity(), capacity);
            assert!((0..capacity).all(|target| references[&0].contains(&target)));
        }

        let mut incoming = HashMap::default();
        incoming.insert(0, (capacity..capacity + 2).collect());
        extend_reference_map(&mut references, incoming);
        assert_eq!(references[&0].len(), capacity + 2);
        assert!((0..capacity + 2).all(|target| references[&0].contains(&target)));
    }

    #[test]
    fn adding_duplicate_references_keeps_set_capacity() {
        fn check<T>(add: impl Fn(&mut SymbolReferences, Word), targets: impl Fn(&SymbolReferences) -> &HashSet<T>)
        where
            T: Eq + Hash,
        {
            let mut references = SymbolReferences::new();
            add(&mut references, word("target0"));
            let capacity = targets(&references).capacity();
            for index in 1..capacity {
                add(&mut references, word(format!("target{index}")));
            }

            assert_eq!(targets(&references).len(), capacity);
            let expected = references.clone();

            for _ in 0..3 {
                for index in 0..capacity {
                    add(&mut references, word(format!("target{index}")));
                }
                assert_eq!(references, expected);
                assert_eq!(targets(&references).capacity(), capacity);
            }

            add(&mut references, word("new_target"));
            assert_eq!(targets(&references).len(), capacity + 1);
        }

        let symbol = (word("source"), empty_word());
        let file = word("file.php");
        for origin in [
            ReferenceOrigin::Symbol(symbol),
            ReferenceOrigin::Symbol((symbol.0, word("method"))),
            ReferenceOrigin::File(file),
        ] {
            for in_signature in [false, true] {
                check(
                    |references, target| references.add_reference(origin, (target, empty_word()), in_signature),
                    |references| match origin {
                        ReferenceOrigin::Symbol(symbol) if in_signature => {
                            &references.symbol_references_to_symbols_in_signature[&symbol]
                        }
                        ReferenceOrigin::Symbol(symbol) => &references.symbol_references_to_symbols[&symbol],
                        ReferenceOrigin::File(file) if in_signature => {
                            &references.file_references_to_symbols_in_signature[&file]
                        }
                        ReferenceOrigin::File(file) => &references.file_references_to_symbols[&file],
                    },
                );
            }

            for read in [false, true] {
                check(
                    |references, target| {
                        let property = (target, word("property"));
                        if read {
                            references.add_property_read_reference(origin, property);
                        } else {
                            references.add_property_write_reference(origin, property);
                        }
                    },
                    |references| match origin {
                        ReferenceOrigin::Symbol(symbol) if read => &references.property_read_references[&symbol],
                        ReferenceOrigin::Symbol(symbol) => &references.property_write_references[&symbol],
                        ReferenceOrigin::File(file) if read => &references.file_property_read_references[&file],
                        ReferenceOrigin::File(file) => &references.file_property_write_references[&file],
                    },
                );
            }
        }

        check(
            |references, target| references.add_overridden_member_reference(symbol, (target, word("method"))),
            |references| &references.symbol_references_to_overridden_members[&symbol],
        );

        check(
            |references, target| references.add_functionlike_return_reference(symbol, (target, empty_word())),
            |references| {
                &references.functionlike_references_to_functionlike_returns[&FunctionLikeIdentifier::Function(symbol.0)]
            },
        );
    }

    #[test]
    fn signatures_dominate_bodies_in_any_insertion_or_merge_order() {
        let origins = [
            ReferenceOrigin::Symbol((word("function"), empty_word())),
            ReferenceOrigin::Symbol((word("class"), word("method"))),
            ReferenceOrigin::File(word("file.php")),
        ];

        for origin in origins {
            for target in [(word("target"), empty_word()), (word("target"), word("member"))] {
                let mut expected = SymbolReferences::new();
                expected.add_reference(origin, target, true);

                for first_is_signature in [false, true] {
                    let mut direct = SymbolReferences::new();
                    direct.add_reference(origin, target, first_is_signature);
                    direct.add_reference(origin, target, !first_is_signature);
                    assert_eq!(direct, expected);

                    let mut merged = SymbolReferences::new();
                    merged.add_reference(origin, target, first_is_signature);
                    let mut incoming = SymbolReferences::new();
                    incoming.add_reference(origin, target, !first_is_signature);
                    merged.extend(incoming);
                    assert_eq!(merged, expected);
                }
            }
        }
    }

    #[test]
    fn signature_precedence_preserves_reference_roles_and_body_restoration() {
        let source = word("source");
        let target = word("target");
        let mut references = SymbolReferences::new();
        add_every_reference_kind(&mut references, source, target);
        let symbol = (source, empty_word());
        let target = (target, empty_word());
        let mut kinds = HashSet::default();
        references.for_each_reference(|origin, referenced, kind| {
            if origin == ReferenceOrigin::Symbol(symbol) && referenced == target {
                kinds.insert(kind);
            }
        });
        assert_eq!(
            kinds,
            HashSet::from_iter([
                SymbolReferenceKind::Signature,
                SymbolReferenceKind::OverriddenMember,
                SymbolReferenceKind::FunctionLikeReturn,
                SymbolReferenceKind::PropertyRead,
                SymbolReferenceKind::PropertyWrite,
            ])
        );

        let mut previous = SymbolReferences::new();
        previous.add_symbol_reference(symbol, target, false);
        references.restore_references_for_safe_symbols(&previous, &WordSet::from_iter([source]), &HashSet::default());
        assert!(!references.symbol_references_to_symbols.contains_key(&symbol));

        references.remove_dirty_symbol_references(&HashSet::from_iter([symbol]));
        references.add_symbol_reference(symbol, target, false);
        assert_eq!(references.symbol_references_to_symbols[&symbol], HashSet::from_iter([target]));
        assert!(!references.symbol_references_to_symbols_in_signature.contains_key(&symbol));
    }

    #[test]
    fn test_for_each_reference_visits_every_reference_kind_and_origin() {
        let function = (word("consumer"), empty_word());
        let signature_function = (word("signature_consumer"), empty_word());
        let class = word("service");
        let method = (class, word("method"));
        let property = (class, word("$property"));
        let file_property = (class, word("$file_property"));
        let override_source = (word("child"), word("method"));
        let return_source = (word("return_consumer"), empty_word());
        let file = word("src/file.php");

        let mut references = SymbolReferences::new();
        assert!(references.is_empty());
        references.add_symbol_reference(function, (class, empty_word()), false);
        references.add_symbol_reference(signature_function, (class, empty_word()), true);
        references.add_property_read_reference(ReferenceOrigin::Symbol(method), property);
        references.add_property_write_reference(ReferenceOrigin::Symbol(function), property);
        references.add_overridden_member_reference(override_source, method);
        references.add_functionlike_return_reference(return_source, method);
        references.add_reference(ReferenceOrigin::File(file), method, false);
        references.add_file_reference_to_class_member(file, property, true);
        references.add_property_read_reference(ReferenceOrigin::File(file), file_property);
        references.add_property_write_reference(ReferenceOrigin::File(file), file_property);
        assert!(!references.is_empty());

        let mut visited = HashSet::default();
        references.for_each_reference(|source, target, kind| {
            visited.insert((source, target, kind));
        });

        assert!(visited.contains(&(
            ReferenceOrigin::Symbol(function),
            (class, empty_word()),
            SymbolReferenceKind::Body
        )));
        assert!(visited.contains(&(
            ReferenceOrigin::Symbol(signature_function),
            (class, empty_word()),
            SymbolReferenceKind::Signature,
        )));
        assert!(visited.contains(&(ReferenceOrigin::Symbol(method), property, SymbolReferenceKind::PropertyRead)));
        assert!(visited.contains(&(ReferenceOrigin::Symbol(function), property, SymbolReferenceKind::PropertyWrite)));
        assert!(visited.contains(&(
            ReferenceOrigin::Symbol(override_source),
            method,
            SymbolReferenceKind::OverriddenMember,
        )));
        assert!(visited.contains(&(
            ReferenceOrigin::Symbol(return_source),
            method,
            SymbolReferenceKind::FunctionLikeReturn,
        )));
        assert!(visited.contains(&(ReferenceOrigin::File(file), method, SymbolReferenceKind::Body)));
        assert!(visited.contains(&(ReferenceOrigin::File(file), property, SymbolReferenceKind::Signature)));
        assert!(visited.contains(&(ReferenceOrigin::File(file), file_property, SymbolReferenceKind::PropertyRead)));
        assert!(visited.contains(&(ReferenceOrigin::File(file), file_property, SymbolReferenceKind::PropertyWrite)));
        assert_eq!(references.count_property_reads(&file_property), 1);
        assert_eq!(references.count_property_writes(&file_property), 1);
    }

    #[test]
    fn test_restore_references_for_safe_symbols_restores_missing_body_refs() {
        let class_a = word("class_a");
        let class_b = word("class_b");
        let method_foo = word("foo");
        let method_bar = word("bar");

        let previous = make_refs_with_body(vec![
            ((class_a, method_foo), vec![(class_b, empty_word())]),
            ((class_b, method_bar), vec![(class_a, empty_word())]),
        ]);

        let mut current = make_refs_with_body(vec![((class_b, method_bar), vec![(class_a, empty_word())])]);

        let safe_symbols = WordSet::default();
        let mut safe_members = HashSet::default();
        safe_members.insert((class_a, method_foo));

        current.restore_references_for_safe_symbols(&previous, &safe_symbols, &safe_members);

        assert!(current.symbol_references_to_symbols.contains_key(&(class_a, method_foo)));
        let restored = &current.symbol_references_to_symbols[&(class_a, method_foo)];
        assert!(restored.contains(&(class_b, empty_word())));

        assert!(current.symbol_references_to_symbols.contains_key(&(class_b, method_bar)));
    }

    #[test]
    fn test_restore_references_does_not_overwrite_existing() {
        let class_a = word("class_a");
        let class_b = word("class_b");
        let class_c = word("class_c");
        let method_foo = word("foo");

        let previous = make_refs_with_body(vec![((class_a, method_foo), vec![(class_b, empty_word())])]);

        let mut current = make_refs_with_body(vec![((class_a, method_foo), vec![(class_c, empty_word())])]);

        let safe_symbols = WordSet::default();
        let mut safe_members = HashSet::default();
        safe_members.insert((class_a, method_foo));

        current.restore_references_for_safe_symbols(&previous, &safe_symbols, &safe_members);

        let refs = &current.symbol_references_to_symbols[&(class_a, method_foo)];
        assert!(refs.contains(&(class_c, empty_word())));
        assert!(!refs.contains(&(class_b, empty_word())));
    }

    #[test]
    fn test_restore_references_for_safe_top_level_symbols() {
        let func_a = word("func_a");
        let class_b = word("class_b");

        let previous = make_refs_with_body(vec![((func_a, empty_word()), vec![(class_b, empty_word())])]);

        let mut current = SymbolReferences::new();

        let mut safe_symbols = WordSet::default();
        safe_symbols.insert(func_a);
        let safe_members = HashSet::default();

        current.restore_references_for_safe_symbols(&previous, &safe_symbols, &safe_members);

        assert!(current.symbol_references_to_symbols.contains_key(&(func_a, empty_word())));
        let restored = &current.symbol_references_to_symbols[&(func_a, empty_word())];
        assert!(restored.contains(&(class_b, empty_word())));
    }

    #[test]
    fn test_restore_skips_non_safe_symbols() {
        let func_a = word("func_a");
        let class_b = word("class_b");
        let previous = make_refs_with_body(vec![((func_a, empty_word()), vec![(class_b, empty_word())])]);

        let mut current = SymbolReferences::new();

        let safe_symbols = WordSet::default();
        let safe_members = HashSet::default();

        current.restore_references_for_safe_symbols(&previous, &safe_symbols, &safe_members);

        assert!(!current.symbol_references_to_symbols.contains_key(&(func_a, empty_word())));
    }

    #[test]
    fn test_get_invalid_symbols_basic_cascade() {
        let class_a = word("class_a");
        let class_b = word("class_b");
        let method_foo = word("foo");

        let mut refs = SymbolReferences::new();
        refs.symbol_references_to_symbols_in_signature.insert((class_b, method_foo), {
            let mut set = HashSet::default();
            set.insert((class_a, empty_word()));
            set
        });

        let mut diff = crate::diff::CodebaseDiff::new();
        let mut changed = HashSet::default();
        changed.insert((class_a, empty_word()));
        diff = diff.with_changed(changed);

        let result = refs.get_invalid_symbols(&diff);
        assert!(result.is_some());
        let (invalid, partially_invalid, invalid_files) = result.unwrap();

        assert!(invalid.contains(&(class_a, empty_word())));
        assert!(invalid.contains(&(class_b, method_foo)));
        assert!(partially_invalid.contains(&class_b));
        assert!(invalid_files.is_empty());
    }

    #[test]
    fn test_get_invalid_symbols_tracks_file_origins() {
        let changed_class = word("changed_class");
        let file = word("src/bootstrap.php");
        let mut references = SymbolReferences::new();
        references.add_reference(ReferenceOrigin::File(file), (changed_class, empty_word()), false);

        let mut diff = crate::diff::CodebaseDiff::new();
        let mut changed = HashSet::default();
        changed.insert((changed_class, empty_word()));
        diff = diff.with_changed(changed);

        let (_, _, invalid_files) = references.get_invalid_symbols(&diff).expect("invalidation should complete");
        assert_eq!(invalid_files.len(), 1);
        assert!(invalid_files.contains(&file));
    }
}
