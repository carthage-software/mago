use ahash::HashMap;
use ahash::HashSet;
use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;

use crate::context::ScopeContext;
use crate::diff::CodebaseDiff;
use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::identifier::method::MethodIdentifier;
use crate::symbol::SymbolIdentifier;

/// Represents the source of a reference, distinguishing between top-level symbols
/// and members within a class-like structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReferenceSource {
    /// A reference from a top-level symbol (function, class, enum, trait, interface, constant).
    /// The bool indicates if the reference occurs within a signature context (true) or body (false).
    /// The StringIdentifier is the name (FQCN or FQN) of the referencing symbol.
    Symbol(bool, StringIdentifier),
    /// A reference from a member within a class-like structure (method, property, class constant, enum case).
    /// The bool indicates if the reference occurs within a signature context (true) or body (false).
    /// The first StringIdentifier is the FQCN of the class-like structure.
    /// The second StringIdentifier is the name of the member.
    ClassLikeMember(bool, StringIdentifier, StringIdentifier),
}

/// Holds sets of symbols and members identified as invalid during analysis,
/// often due to changes detected in `CodebaseDiff`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InvalidSymbols {
    /// Set of (Symbol, Member) pairs whose *signatures* are considered invalid.
    /// An empty member name usually indicates the symbol itself.
    invalid_symbol_and_member_signatures: HashSet<SymbolIdentifier>,
    /// Set of (Symbol, Member) pairs whose *bodies* are considered invalid.
    /// An empty member name usually indicates the symbol itself.
    invalid_symbol_and_member_bodies: HashSet<SymbolIdentifier>,
    /// Set of top-level symbols (class FQCN, function FQN) that are partially invalid,
    /// meaning at least one member's signature or body is invalid, but not necessarily the whole symbol.
    partially_invalid_symbols: HashSet<StringIdentifier>,
}

/// Stores various maps tracking references between symbols (classes, functions, etc.)
/// and class-like members (methods, properties, constants, etc.) within the codebase.
///
/// This is primarily used for dependency analysis, understanding code structure,
/// and potentially for tasks like dead code detection or impact analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)] // Added PartialEq
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
}

impl SymbolReferences {
    /// Creates a new, empty `SymbolReferences` collection.
    #[inline]
    pub fn new() -> Self {
        Self {
            symbol_references_to_symbols: HashMap::default(),
            symbol_references_to_symbols_in_signature: HashMap::default(),
            symbol_references_to_overridden_members: HashMap::default(),
            functionlike_references_to_functionlike_returns: HashMap::default(),
        }
    }

    /// Records that a top-level symbol (e.g., a function) references a class member.
    ///
    /// Automatically adds a reference from the referencing symbol to the member's class.
    ///
    /// # Arguments
    /// * `referencing_symbol`: The FQN of the function or global const making the reference.
    /// * `class_member`: A tuple `(ClassName, MemberName)` being referenced.
    /// * `in_signature`: `true` if the reference occurs in a signature context, `false` if in the body.
    #[inline]
    pub fn add_symbol_reference_to_class_member(
        &mut self,
        referencing_symbol: StringIdentifier,
        class_member: SymbolIdentifier,
        in_signature: bool,
    ) {
        // Reference the class itself implicitly (in body context)
        self.add_symbol_reference_to_symbol(referencing_symbol, class_member.0, false);

        // Use empty member for the referencing symbol key
        let key = (referencing_symbol, StringIdentifier::empty());
        if in_signature {
            self.symbol_references_to_symbols_in_signature.entry(key).or_default().insert(class_member);
        } else {
            self.symbol_references_to_symbols.entry(key).or_default().insert(class_member);
        }
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
    pub fn add_symbol_reference_to_symbol(
        &mut self,
        referencing_symbol: StringIdentifier,
        symbol: StringIdentifier,
        in_signature: bool,
    ) {
        if referencing_symbol == symbol {
            return;
        }

        // Represent top-level symbols with an empty member identifier
        let referencing_key = (referencing_symbol, StringIdentifier::empty());
        let referenced_key = (symbol, StringIdentifier::empty());

        if in_signature {
            self.symbol_references_to_symbols_in_signature.entry(referencing_key).or_default().insert(referenced_key);
        } else {
            // If it's already referenced in the signature, don't add as a body reference
            if let Some(sig_refs) = self.symbol_references_to_symbols_in_signature.get(&referencing_key)
                && sig_refs.contains(&referenced_key)
            {
                return;
            }
            self.symbol_references_to_symbols.entry(referencing_key).or_default().insert(referenced_key);
        }
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
        if in_signature {
            self.symbol_references_to_symbols_in_signature
                .entry(referencing_class_member)
                .or_default()
                .insert(class_member);
        } else {
            // Check signature refs first? (Consistency with add_symbol_reference_to_symbol might be needed)
            // Current logic adds to body refs regardless of signature refs for member->member.
            self.symbol_references_to_symbols.entry(referencing_class_member).or_default().insert(class_member);
        }
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
        symbol: StringIdentifier,
        in_signature: bool,
    ) {
        if referencing_class_member.0 == symbol {
            return;
        }

        // Add implicit reference from the class to the symbol
        self.add_symbol_reference_to_symbol(referencing_class_member.0, symbol, false);

        // Represent the referenced symbol with an empty member identifier
        let referenced_key = (symbol, StringIdentifier::empty());

        if in_signature {
            self.symbol_references_to_symbols_in_signature
                .entry(referencing_class_member)
                .or_default()
                .insert(referenced_key);
        } else {
            // If already referenced in signature, don't add as body reference
            if let Some(sig_refs) = self.symbol_references_to_symbols_in_signature.get(&referencing_class_member)
                && sig_refs.contains(&referenced_key)
            {
                return;
            }
            self.symbol_references_to_symbols.entry(referencing_class_member).or_default().insert(referenced_key);
        }
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
        if let Some(referencing_functionlike) = scope.get_function_like_identifier() {
            match referencing_functionlike {
                FunctionLikeIdentifier::Function(function_name) => {
                    self.add_symbol_reference_to_class_member(function_name, class_member, in_signature)
                }
                FunctionLikeIdentifier::Method(class_name, function_name) => self
                    .add_class_member_reference_to_class_member(
                        (class_name, function_name),
                        class_member,
                        in_signature,
                    ),
                _ => {
                    // A reference from a closure can be ignored for now.
                }
            }
        } else if let Some(calling_class) = scope.get_class_like_name() {
            // Reference from the class scope itself (e.g., property default)
            self.add_symbol_reference_to_class_member(*calling_class, class_member, in_signature)
        }
        // If no context, the reference source is unknown/untracked in this map
    }

    #[inline]
    pub fn add_reference_for_method_call(&mut self, scope: &ScopeContext<'_>, method: &MethodIdentifier) {
        self.add_reference_to_class_member(scope, (*method.get_class_name(), *method.get_method_name()), false);
    }

    #[inline]
    pub fn add_reference_for_property_access(
        &mut self,
        scope: &ScopeContext<'_>,
        class_name: StringIdentifier,
        property_name: StringIdentifier,
    ) {
        self.add_reference_to_class_member(scope, (class_name, property_name), false);
    }

    /// Convenience method to add a reference *from* the current function context *to* an overridden class member (e.g., `parent::foo`).
    /// Delegates based on the function context.
    #[inline]
    pub fn add_reference_to_overridden_class_member(&mut self, scope: &ScopeContext, class_member: SymbolIdentifier) {
        let referencing_key = if let Some(referencing_functionlike) = scope.get_function_like_identifier() {
            match referencing_functionlike {
                FunctionLikeIdentifier::Function(function_name) => (StringIdentifier::empty(), function_name),
                FunctionLikeIdentifier::Method(class_name, function_name) => (class_name, function_name),
                _ => {
                    // A reference from a closure can be ignored for now.
                    return;
                }
            }
        } else if let Some(calling_class) = scope.get_class_like_name() {
            (*calling_class, StringIdentifier::empty())
        } else {
            return; // Cannot record reference without a source context
        };

        self.symbol_references_to_overridden_members.entry(referencing_key).or_default().insert(class_member);
    }

    /// Convenience method to add a reference *from* the current function context *to* a top-level symbol.
    /// Delegates to appropriate `add_*` methods based on the function context.
    #[inline]
    pub fn add_reference_to_symbol(&mut self, scope: &ScopeContext, symbol: StringIdentifier, in_signature: bool) {
        if let Some(referencing_functionlike) = scope.get_function_like_identifier() {
            match referencing_functionlike {
                FunctionLikeIdentifier::Function(function_name) => {
                    self.add_symbol_reference_to_symbol(function_name, symbol, in_signature)
                }
                FunctionLikeIdentifier::Method(class_name, function_name) => {
                    self.add_class_member_reference_to_symbol((class_name, function_name), symbol, in_signature)
                }
                _ => {
                    // Ignore references from closures.
                }
            }
        } else if let Some(calling_class) = scope.get_class_like_name() {
            self.add_symbol_reference_to_symbol(*calling_class, symbol, in_signature)
        }
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

        self.functionlike_references_to_functionlike_returns
            .entry(referencing_functionlike)
            .or_default()
            .insert(referenced_functionlike);
    }

    /// Merges references from another `SymbolReferences` instance into this one.
    /// Existing references are extended, not replaced.
    #[inline]
    pub fn extend(&mut self, other: Self) {
        for (k, v) in other.symbol_references_to_symbols {
            self.symbol_references_to_symbols.entry(k).or_default().extend(v);
        }
        for (k, v) in other.symbol_references_to_symbols_in_signature {
            self.symbol_references_to_symbols_in_signature.entry(k).or_default().extend(v);
        }
        for (k, v) in other.symbol_references_to_overridden_members {
            self.symbol_references_to_overridden_members.entry(k).or_default().extend(v);
        }
        for (k, v) in other.functionlike_references_to_functionlike_returns {
            self.functionlike_references_to_functionlike_returns.entry(k).or_default().extend(v);
        }
    }

    /// Computes the set of all unique symbols and members that are referenced *by* any symbol/member
    /// tracked in the body or signature reference maps.
    ///
    /// # Returns
    ///
    /// A `HashSet` containing `&(SymbolName, MemberName)` tuples of all referenced items.
    #[inline]
    pub fn get_referenced_symbols_and_members(&self) -> HashSet<&SymbolIdentifier> {
        let mut referenced_items = HashSet::default();
        for refs in self.symbol_references_to_symbols.values() {
            referenced_items.extend(refs.iter());
        }
        for refs in self.symbol_references_to_symbols_in_signature.values() {
            referenced_items.extend(refs.iter());
        }

        referenced_items
    }

    /// Computes the inverse of the body and signature reference maps.
    ///
    /// # Returns
    ///
    /// A `HashMap` where the key is the referenced symbol/member `(Symbol, Member)` and the value
    /// is a `HashSet` of referencing symbols/members `(RefSymbol, RefMember)`.
    #[inline]
    pub fn get_back_references(&self) -> HashMap<SymbolIdentifier, HashSet<SymbolIdentifier>> {
        let mut back_refs: HashMap<SymbolIdentifier, HashSet<SymbolIdentifier>> = HashMap::default();

        for (referencing_item, referenced_items) in &self.symbol_references_to_symbols {
            for referenced_item in referenced_items {
                back_refs.entry(*referenced_item).or_default().insert(*referencing_item);
            }
        }
        for (referencing_item, referenced_items) in &self.symbol_references_to_symbols_in_signature {
            for referenced_item in referenced_items {
                back_refs.entry(*referenced_item).or_default().insert(*referencing_item);
            }
        }
        back_refs
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
    pub fn get_references_to_symbol(&self, target_symbol: SymbolIdentifier) -> HashSet<&SymbolIdentifier> {
        let mut referencing_items = HashSet::default();
        for (referencing_item, referenced_items) in &self.symbol_references_to_symbols {
            if referenced_items.contains(&target_symbol) {
                referencing_items.insert(referencing_item);
            }
        }
        for (referencing_item, referenced_items) in &self.symbol_references_to_symbols_in_signature {
            if referenced_items.contains(&target_symbol) {
                referencing_items.insert(referencing_item);
            }
        }
        referencing_items
    }

    /// Computes the count of references for each unique symbol/member referenced in bodies or signatures.
    ///
    /// # Returns
    ///
    /// A `HashMap` where the key is the referenced symbol/member `(Symbol, Member)` and the value
    /// is the total count (`u32`) of references to it.
    #[inline]
    pub fn get_referenced_symbols_and_members_with_counts(&self) -> HashMap<SymbolIdentifier, u32> {
        let mut counts = HashMap::default();
        for referenced_items in self.symbol_references_to_symbols.values() {
            for referenced_item in referenced_items {
                *counts.entry(*referenced_item).or_insert(0) += 1;
            }
        }
        for referenced_items in self.symbol_references_to_symbols_in_signature.values() {
            for referenced_item in referenced_items {
                *counts.entry(*referenced_item).or_insert(0) += 1;
            }
        }
        counts
    }

    /// Computes the inverse of the overridden member reference map.
    ///
    /// # Returns
    ///
    /// A `HashMap` where the key is the overridden member `(ParentSymbol, Member)` and the value
    /// is a `HashSet` of referencing symbols/members `(RefSymbol, RefMember)` that call it via `parent::`.
    #[inline]
    pub fn get_referenced_overridden_class_members(&self) -> HashMap<SymbolIdentifier, HashSet<SymbolIdentifier>> {
        let mut back_refs: HashMap<SymbolIdentifier, HashSet<SymbolIdentifier>> = HashMap::default();

        for (referencing_item, referenced_items) in &self.symbol_references_to_overridden_members {
            for referenced_item in referenced_items {
                back_refs.entry(*referenced_item).or_default().insert(*referencing_item);
            }
        }
        back_refs
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
    /// `Some((invalid_signatures, partially_invalid))` on success, where `invalid_signatures` contains
    /// all symbol/member pairs whose signature is invalid (including propagated ones), and `partially_invalid`
    /// contains symbols with at least one invalid member.
    /// Returns `None` if the propagation exceeds an expense limit (currently 5000 steps).
    #[inline]
    pub fn get_invalid_symbols(
        &self,
        codebase_diff: &CodebaseDiff,
    ) -> Option<(HashSet<SymbolIdentifier>, HashSet<StringIdentifier>)> {
        let mut invalid_signatures = HashSet::default();
        let mut partially_invalid_symbols = HashSet::default();

        for sig_ref_key in self.symbol_references_to_symbols_in_signature.keys() {
            // Represent the containing symbol (ignore member part for diff check)
            let containing_symbol = (sig_ref_key.0, StringIdentifier::empty());

            if codebase_diff.contains_add_or_delete_entry(&containing_symbol) {
                invalid_signatures.insert(*sig_ref_key);
                partially_invalid_symbols.insert(sig_ref_key.0);
            }
        }

        // Start with symbols directly added/deleted in the diff.
        let mut symbols_to_process = codebase_diff.get_add_or_delete().iter().copied().collect::<Vec<_>>();
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
                // If it's a member...
                partially_invalid_symbols.insert(invalidated_item.0);
            }

            // Find all items that reference this now-invalid item *in their signature*
            for (referencing_item, referenced_items) in &self.symbol_references_to_symbols_in_signature {
                if referenced_items.contains(&invalidated_item) {
                    // If referencing item not already processed, add it to the processing queue
                    if !processed_symbols.contains(referencing_item) {
                        symbols_to_process.push(*referencing_item);
                    }

                    // Mark the referencing item itself as invalid (signature)
                    invalid_signatures.insert(*referencing_item);
                    if !referencing_item.1.is_empty() {
                        // If it's a member...
                        partially_invalid_symbols.insert(referencing_item.0);
                    }
                }
            }

            // Simple check against limit within loop might be slightly faster
            if expense_counter > EXPENSE_LIMIT {
                return None;
            }
        }

        // An item's body is invalid if it references (anywhere, body or sig) an item with an invalid signature,
        // OR if its own signature was kept but its body might have changed (keep_signature diff).
        let mut invalid_bodies = HashSet::default();

        // Check references from body map
        for (referencing_item, referenced_items) in &self.symbol_references_to_symbols {
            // Does this item reference *any* item with an invalid signature?
            if referenced_items.iter().any(|r| invalid_signatures.contains(r)) {
                invalid_bodies.insert(*referencing_item);
                if !referencing_item.1.is_empty() {
                    // If it's a member...
                    partially_invalid_symbols.insert(referencing_item.0);
                }
            }
        }

        // Check references from signature map (redundant with propagation? Maybe not entirely)
        // If item A's signature references item B (invalid signature), A's signature becomes invalid (handled above).
        // But A's *body* might also be considered invalid due to the signature dependency.
        for (referencing_item, referenced_items) in &self.symbol_references_to_symbols_in_signature {
            if referenced_items.iter().any(|r| invalid_signatures.contains(r)) {
                invalid_bodies.insert(*referencing_item);
                if !referencing_item.1.is_empty() {
                    partially_invalid_symbols.insert(referencing_item.0);
                }
            }
        }

        // Add items whose signatures were kept, but bodies might have changed
        for keep_sig_item in codebase_diff.get_keep_signature() {
            invalid_bodies.insert(*keep_sig_item);
            if !keep_sig_item.1.is_empty() {
                partially_invalid_symbols.insert(keep_sig_item.0);
            }
        }

        // Combine results: invalid_signatures includes items whose definition changed or depend on changed signatures.
        // partially_invalid_symbols includes symbols containing members from either invalid_signatures or invalid_bodies.
        Some((invalid_signatures, partially_invalid_symbols))
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
    }

    /// Returns a reference to the map tracking references within symbol/member bodies.
    #[inline]
    pub fn get_symbol_references_to_symbols(&self) -> &HashMap<SymbolIdentifier, HashSet<SymbolIdentifier>> {
        &self.symbol_references_to_symbols
    }

    /// Returns a reference to the map tracking references within symbol/member signatures.
    #[inline]
    pub fn get_symbol_references_to_symbols_in_signature(
        &self,
    ) -> &HashMap<SymbolIdentifier, HashSet<SymbolIdentifier>> {
        &self.symbol_references_to_symbols_in_signature
    }

    /// Returns a reference to the map tracking references to overridden members.
    #[inline]
    pub fn get_symbol_references_to_overridden_members(&self) -> &HashMap<SymbolIdentifier, HashSet<SymbolIdentifier>> {
        &self.symbol_references_to_overridden_members
    }

    /// Returns a reference to the map tracking references to function-like return values.
    #[inline]
    pub fn get_functionlike_references_to_functionlike_returns(
        &self,
    ) -> &HashMap<FunctionLikeIdentifier, HashSet<FunctionLikeIdentifier>> {
        &self.functionlike_references_to_functionlike_returns
    }
}
