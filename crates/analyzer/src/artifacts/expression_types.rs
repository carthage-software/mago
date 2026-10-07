use std::rc::Rc;

use foldhash::HashMap;
use mago_codex::ttype::union::TUnion;

type ExpressionRange = (u32, u32);
type TypeMap = HashMap<ExpressionRange, Rc<TUnion>>;

/// Nested checkpoints that copy only changed expression types.
#[derive(Debug, Clone, Default)]
pub(crate) struct ExpressionTypeCheckpoints {
    checkpoints: Vec<Checkpoint>,
}

#[derive(Debug, Clone)]
enum Checkpoint {
    Changes(HashMap<ExpressionRange, Option<Rc<TUnion>>>),
    Snapshot(TypeMap),
}

#[derive(Clone, Copy)]
pub(crate) enum CheckpointAction {
    /// Keep all changes, including removals.
    Keep,
    /// Restore the map from before the checkpoint.
    Rollback,
    /// Keep changes, but restore removed entries from before the checkpoint.
    Merge,
}

impl ExpressionTypeCheckpoints {
    pub(crate) fn is_empty(&self) -> bool {
        self.checkpoints.is_empty()
    }

    pub(crate) fn begin_checkpoint(&mut self) {
        self.checkpoints.push(Checkpoint::Changes(HashMap::default()));
    }

    pub(crate) fn checkpoint_matches(&self, types: &TypeMap, same: impl Fn(&TUnion, &TUnion) -> bool) -> bool {
        let Some(checkpoint) = self.checkpoints.last() else {
            unreachable!("expression type checkpoint must exist");
        };
        match checkpoint {
            Checkpoint::Changes(changes) => {
                changes.iter().all(|(range, previous)| match (previous, types.get(range)) {
                    (Some(previous), Some(current)) => same(previous, current),
                    (None, None) => true,
                    _ => false,
                })
            }
            Checkpoint::Snapshot(snapshot) => {
                snapshot.len() == types.len()
                    && snapshot
                        .iter()
                        .all(|(range, previous)| types.get(range).is_some_and(|current| same(previous, current)))
            }
        }
    }

    pub(crate) fn finish_checkpoint(&mut self, types: &mut TypeMap, action: CheckpointAction) {
        let Some(checkpoint) = self.checkpoints.pop() else {
            unreachable!("expression type checkpoint must exist");
        };

        let changes = match checkpoint {
            Checkpoint::Changes(changes) => changes,
            Checkpoint::Snapshot(mut snapshot) => {
                match action {
                    CheckpointAction::Keep => {}
                    CheckpointAction::Rollback => *types = snapshot,
                    CheckpointAction::Merge => {
                        snapshot.extend(std::mem::take(types));
                        *types = snapshot;
                    }
                }

                return;
            }
        };

        if matches!(action, CheckpointAction::Rollback) {
            restore_changes(types, changes);
            return;
        }

        for (range, previous_type) in changes {
            if matches!(action, CheckpointAction::Merge)
                && let Some(previous_type) = &previous_type
            {
                types.entry(range).or_insert_with(|| Rc::clone(previous_type));
            }

            if let Some(Checkpoint::Changes(parent)) = self.checkpoints.last_mut() {
                parent.entry(range).or_insert(previous_type);
            }
        }
    }

    /// Preserve complete maps before a hook makes untracked changes.
    pub(crate) fn detach(&mut self, types: &TypeMap) -> Self {
        let mut previous_types = types.clone();
        let mut checkpoints = Vec::with_capacity(self.checkpoints.len());
        for checkpoint in std::mem::take(&mut self.checkpoints).into_iter().rev() {
            match checkpoint {
                Checkpoint::Changes(changes) => restore_changes(&mut previous_types, changes),
                Checkpoint::Snapshot(snapshot) => previous_types = snapshot,
            }
            checkpoints.push(Checkpoint::Snapshot(previous_types.clone()));
        }
        checkpoints.reverse();

        Self { checkpoints }
    }

    #[inline]
    pub(crate) fn insert(&mut self, types: &mut TypeMap, range: ExpressionRange, ty: Rc<TUnion>) -> Option<Rc<TUnion>> {
        let previous_type = types.insert(range, ty);
        if let Some(Checkpoint::Changes(changes)) = self.checkpoints.last_mut() {
            changes.entry(range).or_insert_with(|| previous_type.clone());
        }

        previous_type
    }

    #[inline]
    pub(crate) fn remove(&mut self, types: &mut TypeMap, range: &ExpressionRange) -> Option<Rc<TUnion>> {
        let previous_type = types.remove(range);
        if let Some(Checkpoint::Changes(changes)) = self.checkpoints.last_mut() {
            changes.entry(*range).or_insert_with(|| previous_type.clone());
        }

        previous_type
    }
}

fn restore_changes(types: &mut TypeMap, changes: HashMap<ExpressionRange, Option<Rc<TUnion>>>) {
    for (range, previous_type) in changes {
        match previous_type {
            Some(previous_type) => {
                types.insert(range, previous_type);
            }
            None => {
                types.remove(&range);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::ops::Deref;
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    use foldhash::HashSet;
    use mago_allocator::LocalArena;
    use mago_codex::context::ScopeContext;
    use mago_codex::metadata::CodebaseMetadata;
    use mago_codex::populator::populate_codebase;
    use mago_codex::reference::ReferenceOrigin;
    use mago_codex::reference::SymbolReferences;
    use mago_codex::scanner::scan_program;
    use mago_codex::ttype::get_literal_int;
    use mago_database::file::File;
    use mago_names::resolver::NameResolver;
    use mago_syntax::cst::Expression;
    use mago_syntax::cst::Literal;
    use mago_syntax::parser::parse_file;
    use mago_word::WordSet;
    use mago_word::word;

    use super::*;
    use crate::Analyzer;
    use crate::analysis_result::AnalysisResult;
    use crate::artifacts::AnalysisArtifacts;
    use crate::context::block::BlockContext;
    use crate::plugin::PluginError;
    use crate::plugin::PluginRegistry;
    use crate::plugin::context::HookContext;
    use crate::plugin::hook::ExpressionHook;
    use crate::plugin::hook::HookResult;
    use crate::plugin::provider::Provider;
    use crate::plugin::provider::ProviderMeta;
    use crate::settings::Settings;

    #[derive(Default)]
    struct ExpressionTypes {
        types: TypeMap,
        checkpoints: ExpressionTypeCheckpoints,
    }

    impl ExpressionTypes {
        fn begin_checkpoint(&mut self) {
            self.checkpoints.begin_checkpoint();
        }

        fn finish_checkpoint(&mut self, action: CheckpointAction) {
            self.checkpoints.finish_checkpoint(&mut self.types, action);
        }

        fn insert(&mut self, range: ExpressionRange, ty: Rc<TUnion>) {
            self.checkpoints.insert(&mut self.types, range, ty);
        }

        fn remove(&mut self, range: &ExpressionRange) {
            self.checkpoints.remove(&mut self.types, range);
        }

        fn extend(&mut self, entries: impl IntoIterator<Item = (ExpressionRange, Rc<TUnion>)>) {
            for (range, ty) in entries {
                self.insert(range, ty);
            }
        }
    }

    impl Deref for ExpressionTypes {
        type Target = TypeMap;

        fn deref(&self) -> &Self::Target {
            &self.types
        }
    }

    fn assert_same_types(actual: &TypeMap, expected: &TypeMap) {
        assert_eq!(actual.len(), expected.len());
        for (range, expected_type) in expected {
            assert!(actual.get(range).is_some_and(|actual_type| Rc::ptr_eq(actual_type, expected_type)));
        }
    }

    fn finish_snapshot(types: &mut TypeMap, mut snapshot: TypeMap, action: CheckpointAction) {
        match action {
            CheckpointAction::Keep => {}
            CheckpointAction::Rollback => *types = snapshot,
            CheckpointAction::Merge => {
                snapshot.extend(std::mem::take(types));
                *types = snapshot;
            }
        }
    }

    #[test]
    fn merge_restores_first_value_after_writes_and_removal() {
        let mut types = ExpressionTypes::default();
        let original_type = Rc::new(get_literal_int(1));
        types.insert((0, 1), Rc::clone(&original_type));
        types.begin_checkpoint();
        types.insert((0, 1), Rc::new(get_literal_int(2)));
        types.remove(&(0, 1));
        types.insert((2, 3), Rc::new(get_literal_int(3)));
        types.remove(&(2, 3));
        types.finish_checkpoint(CheckpointAction::Merge);

        assert_same_types(&types, &HashMap::from_iter([((0, 1), original_type)]));
        assert!(types.checkpoints.is_empty());
    }

    #[test]
    fn nested_checkpoints_match_full_map_snapshots() {
        let actions = [CheckpointAction::Keep, CheckpointAction::Rollback, CheckpointAction::Merge];
        for outer_action in actions {
            for inner_action in actions {
                let mut types = ExpressionTypes::default();
                types.extend((0..4).map(|key| ((key, key + 1), Rc::new(get_literal_int(i64::from(key))))));
                let mut expected = types.types.clone();
                let outer_snapshot = expected.clone();
                types.begin_checkpoint();

                let changed_type = Rc::new(get_literal_int(10));
                types.insert((0, 1), Rc::clone(&changed_type));
                expected.insert((0, 1), changed_type);
                types.remove(&(1, 2));
                expected.remove(&(1, 2));

                let inner_snapshot = expected.clone();
                types.begin_checkpoint();
                types.remove(&(0, 1));
                expected.remove(&(0, 1));
                types.remove(&(2, 3));
                expected.remove(&(2, 3));
                let additions = [(1, 2), (4, 5)].map(|range| (range, Rc::new(get_literal_int(20))));
                types.extend(additions.clone());
                expected.extend(additions);

                types.finish_checkpoint(inner_action);
                finish_snapshot(&mut expected, inner_snapshot, inner_action);
                assert_same_types(&types, &expected);
                types.remove(&(3, 4));
                expected.remove(&(3, 4));
                types.finish_checkpoint(outer_action);
                finish_snapshot(&mut expected, outer_snapshot, outer_action);
                assert_same_types(&types, &expected);
                assert!(types.checkpoints.is_empty());
            }
        }
    }

    #[test]
    fn mixed_operations_match_full_map_snapshots() {
        let mut types = ExpressionTypes::default();
        let mut expected = TypeMap::default();
        let mut snapshots = Vec::new();
        let mut random = 17u64;

        for step in 0..10_000 {
            random ^= random << 13;
            random ^= random >> 7;
            random ^= random << 17;
            let key = ((random >> 8) % 11) as u32;
            let range = (key, key + 1);

            match random % 7 {
                0 | 1 => {
                    let ty = Rc::new(get_literal_int(step));
                    types.insert(range, Rc::clone(&ty));
                    expected.insert(range, ty);
                }
                2 => {
                    types.remove(&range);
                    expected.remove(&range);
                }
                3 if snapshots.len() < 6 => {
                    types.begin_checkpoint();
                    snapshots.push(expected.clone());
                }
                action @ 4..=6 if !snapshots.is_empty() => {
                    let action = match action {
                        4 => CheckpointAction::Keep,
                        5 => CheckpointAction::Rollback,
                        _ => CheckpointAction::Merge,
                    };
                    types.finish_checkpoint(action);
                    let Some(snapshot) = snapshots.pop() else {
                        unreachable!("snapshot must exist");
                    };
                    finish_snapshot(&mut expected, snapshot, action);
                }
                _ => {}
            }

            assert_same_types(&types, &expected);
        }
    }

    #[test]
    fn cloned_artifacts_keep_independent_active_checkpoints() {
        let mut artifacts = AnalysisArtifacts::new();
        let original_type = Rc::new(get_literal_int(1));
        artifacts.expression_types.insert((0, 1), Rc::clone(&original_type));
        artifacts.begin_expression_type_checkpoint();
        let changed_type = Rc::new(get_literal_int(2));
        artifacts.extend_expression_types([((0, 1), Rc::clone(&changed_type))]);

        let mut speculative_artifacts = artifacts.clone();
        speculative_artifacts.remove_expression_type(&(0, 1));
        speculative_artifacts.finish_expression_type_checkpoint(CheckpointAction::Merge);
        assert_same_types(&speculative_artifacts.expression_types, &HashMap::from_iter([((0, 1), original_type)]));

        artifacts.finish_expression_type_checkpoint(CheckpointAction::Keep);
        assert_same_types(&artifacts.expression_types, &HashMap::from_iter([((0, 1), changed_type)]));
        assert!(artifacts.expression_type_checkpoints.is_empty());
        assert!(speculative_artifacts.expression_type_checkpoints.is_empty());
    }

    #[test]
    fn failed_operation_keeps_changes_for_parent_checkpoint() {
        let mut types = ExpressionTypes::default();
        let original_type = Rc::new(get_literal_int(1));
        types.insert((0, 1), Rc::clone(&original_type));
        types.begin_checkpoint();
        types.begin_checkpoint();
        let result: Result<(), ()> = {
            types.remove(&(0, 1));
            types.insert((2, 3), Rc::new(get_literal_int(2)));
            Err(())
        };
        types.finish_checkpoint(if result.is_ok() { CheckpointAction::Merge } else { CheckpointAction::Keep });
        assert!(!types.contains_key(&(0, 1)));
        assert!(types.contains_key(&(2, 3)));
        types.finish_checkpoint(CheckpointAction::Rollback);
        assert_same_types(&types, &HashMap::from_iter([((0, 1), original_type)]));
        assert!(types.checkpoints.is_empty());
    }

    #[derive(Clone, Copy, Debug)]
    enum Mutation {
        Clear,
        Take,
        ReplaceMap,
        ReplaceArtifacts,
        EditEntries,
    }

    impl Mutation {
        fn apply(self, artifacts: &mut AnalysisArtifacts) {
            match self {
                Self::Clear => artifacts.expression_types.clear(),
                Self::Take => drop(std::mem::take(&mut artifacts.expression_types)),
                Self::ReplaceMap => artifacts.expression_types = HashMap::default(),
                Self::ReplaceArtifacts => *artifacts = AnalysisArtifacts::new(),
                Self::EditEntries => {
                    artifacts.expression_types.entry((4, 5)).or_insert_with(|| Rc::new(get_literal_int(4)));
                    if let Some(ty) = artifacts.expression_types.get_mut(&(0, 1)) {
                        *ty = Rc::new(get_literal_int(10));
                    }
                    artifacts.expression_types.retain(|range, _| range.0 != 2);
                }
            }
        }
    }

    #[test]
    fn opaque_hook_mutations_match_nested_snapshots() {
        let codebase = CodebaseMetadata::new();
        let file = File::ephemeral(Cow::Borrowed(b"hook.php"), Cow::Borrowed(b"<?php"));
        let mut block = BlockContext::new(ScopeContext::new(ReferenceOrigin::File(word(b"hook.php"))), false);
        let actions = [CheckpointAction::Keep, CheckpointAction::Rollback, CheckpointAction::Merge];
        let mutations =
            [Mutation::Clear, Mutation::Take, Mutation::ReplaceMap, Mutation::ReplaceArtifacts, Mutation::EditEntries];

        for mutation in mutations {
            for outer_action in actions {
                for inner_action in actions {
                    for fail in [false, true] {
                        let mut artifacts = AnalysisArtifacts::new();
                        artifacts.extend_expression_types(
                            (0..3).map(|key| ((key, key + 1), Rc::new(get_literal_int(i64::from(key))))),
                        );
                        let outer_snapshot = artifacts.expression_types.clone();
                        artifacts.begin_expression_type_checkpoint();
                        artifacts.extend_expression_types([((0, 1), Rc::new(get_literal_int(10)))]);
                        let inner_snapshot = artifacts.expression_types.clone();
                        artifacts.begin_expression_type_checkpoint();
                        artifacts.remove_expression_type(&(1, 2));

                        let result: Result<(), ()> = (|| {
                            let mut context = HookContext::new(&codebase, &file, &mut block, &mut artifacts);
                            mutation.apply(context.artifacts_mut());
                            context.artifacts_mut().expression_types.insert((9, 10), Rc::new(get_literal_int(9)));
                            if fail {
                                return Err(());
                            }
                            assert!(context.take_issues().is_empty());
                            Ok(())
                        })();
                        assert_eq!(result.is_err(), fail);
                        let mut expected = artifacts.expression_types.clone();
                        artifacts.finish_expression_type_checkpoint(inner_action);
                        finish_snapshot(&mut expected, inner_snapshot, inner_action);
                        assert_same_types(&artifacts.expression_types, &expected);
                        artifacts.finish_expression_type_checkpoint(outer_action);
                        finish_snapshot(&mut expected, outer_snapshot, outer_action);
                        assert_same_types(&artifacts.expression_types, &expected);
                        assert!(artifacts.expression_type_checkpoints.is_empty());
                    }
                }
            }
        }
    }

    struct MutatingHook {
        mutation: Mutation,
        fail: bool,
        calls: Arc<AtomicUsize>,
    }

    impl Provider for MutatingHook {
        fn meta() -> &'static ProviderMeta {
            static META: ProviderMeta =
                ProviderMeta::new("checkpoint-test", "Checkpoint test", "Tests mutable artifacts");
            &META
        }
    }

    impl ExpressionHook for MutatingHook {
        fn after_expression(&self, expression: &Expression<'_>, context: &mut HookContext<'_, '_>) -> HookResult<()> {
            if matches!(expression, Expression::Literal(Literal::Integer(value)) if value.value == Some(777))
                && !context.artifacts().expression_type_checkpoints.is_empty()
                && self.calls.fetch_add(1, Ordering::SeqCst) == 0
            {
                self.mutation.apply(context.artifacts_mut());
                if self.fail {
                    return Err(PluginError::Internal { reason: "checkpoint test".to_owned() });
                }
            }
            Ok(())
        }
    }

    #[test]
    fn mutable_hooks_work_in_switch_assignment_and_if_checkpoints() {
        let bodies = [
            "$before = 1; switch ($flag) { case true: $value = 777; break; default: break; }",
            "$before = 1; $value = 2; $value += 777;",
            "$before = 1; if ($flag || ($value = 777)) { return; }",
        ];
        let mutations =
            [Mutation::Clear, Mutation::Take, Mutation::ReplaceMap, Mutation::ReplaceArtifacts, Mutation::EditEntries];

        for body in bodies {
            for mutation in mutations {
                for fail in [false, true] {
                    let arena = LocalArena::new();
                    let code = format!("<?php function example(bool $flag): void {{ {body} }}");
                    let file = File::ephemeral(Cow::Borrowed(b"hook.php"), Cow::Owned(code.into_bytes()));
                    let program = parse_file(&arena, &file);
                    assert!(!program.has_errors());
                    let resolved_names = NameResolver::new(&arena).resolve(program);
                    let settings = Settings::default();
                    let mut codebase = scan_program(&arena, &file, program, &resolved_names, settings.version);
                    let mut references = SymbolReferences::new();
                    populate_codebase(&mut codebase, &mut references, WordSet::default(), HashSet::default());
                    let mut registry = PluginRegistry::with_library_providers();
                    let calls = Arc::new(AtomicUsize::new(0));
                    registry.register_expression_hook(MutatingHook { mutation, fail, calls: Arc::clone(&calls) });
                    let analyzer = Analyzer::new(&arena, &file, &resolved_names, &codebase, &registry, settings);
                    let result = analyzer.analyze(program, &mut AnalysisResult::new(references));

                    assert!(calls.load(Ordering::SeqCst) > 0, "hook did not reach checkpoint: {body}");
                    assert_eq!(result.is_err(), fail, "{mutation:?}: {body}");
                }
            }
        }
    }
}
