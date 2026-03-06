use std::collections::BTreeMap;
use std::collections::HashSet;

use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;
use mago_syntax::ast::Binary;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::ClassConstantAccess;
use mago_syntax::ast::ConstantAccess;
use mago_syntax::ast::Expression;
use mago_syntax::ast::FunctionCall;
use mago_syntax::ast::Hint;
use mago_syntax::ast::Identifier;
use mago_syntax::ast::Instantiation;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Statement;
use mago_syntax::ast::StaticMethodCall;
use mago_syntax::ast::StaticPropertyAccess;
use mago_syntax::walker::MutWalker;
use mago_syntax::walker::walk_program_mut;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

/// The kind of a fully-qualified name reference, determined by AST context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NameKind {
    Class,
    Function,
    Constant,
}

/// Collected info about a fully-qualified global name reference.
#[derive(Debug)]
struct FqnReference {
    /// The short name (without leading backslash), e.g. "DateTime"
    name: String,
    /// The span covering the full `\DateTime` in source
    span: Span,
    /// Whether this is a class, function, or constant reference
    kind: NameKind,
}

/// Walker that visits the AST and collects FullyQualified identifiers with their context kind.
struct FqnCollector {
    refs: Vec<FqnReference>,
}

impl FqnCollector {
    fn new() -> Self {
        Self { refs: Vec::new() }
    }

    fn add_from_identifier(&mut self, id: &Identifier<'_>, kind: NameKind) {
        if let Identifier::FullyQualified(fq) = id {
            let value = fq.value.trim_start_matches('\\');
            // Only global namespace names (no backslash in the resolved part)
            if !value.contains('\\') {
                self.refs.push(FqnReference { name: value.to_string(), span: fq.span, kind });
            }
        }
    }

    fn add_from_expression(&mut self, expr: &Expression<'_>, kind: NameKind) {
        if let Expression::Identifier(id) = expr {
            self.add_from_identifier(id, kind);
        }
    }
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for FqnCollector {
    // Function calls: \strlen(...) → Function kind
    fn walk_in_function_call(&mut self, function_call: &'ast FunctionCall<'arena>, _: &mut ()) {
        self.add_from_expression(function_call.function, NameKind::Function);
    }

    // new \DateTime() → Class kind
    fn walk_in_instantiation(&mut self, instantiation: &'ast Instantiation<'arena>, _: &mut ()) {
        self.add_from_expression(instantiation.class, NameKind::Class);
    }

    // \DateTime::createFromFormat() → Class kind
    fn walk_in_static_method_call(
        &mut self,
        static_method_call: &'ast StaticMethodCall<'arena>,
        _: &mut (),
    ) {
        self.add_from_expression(static_method_call.class, NameKind::Class);
    }

    // \SomeClass::$prop → Class kind
    fn walk_in_static_property_access(
        &mut self,
        static_property_access: &'ast StaticPropertyAccess<'arena>,
        _: &mut (),
    ) {
        self.add_from_expression(static_property_access.class, NameKind::Class);
    }

    // \SomeClass::CONST → Class kind
    fn walk_in_class_constant_access(
        &mut self,
        class_constant_access: &'ast ClassConstantAccess<'arena>,
        _: &mut (),
    ) {
        self.add_from_expression(class_constant_access.class, NameKind::Class);
    }

    // Type hints: \DateTime $d, function(): \DateTime, catch (\Exception) → Class kind
    fn walk_in_hint(&mut self, hint: &'ast Hint<'arena>, _: &mut ()) {
        if let Hint::Identifier(id) = hint {
            self.add_from_identifier(id, NameKind::Class);
        }
    }

    // instanceof \DateTime → Class kind (rhs of instanceof binary)
    fn walk_in_binary(&mut self, binary: &'ast Binary<'arena>, _: &mut ()) {
        if matches!(binary.operator, BinaryOperator::Instanceof(_)) {
            self.add_from_expression(binary.rhs, NameKind::Class);
        }
    }

    // \PHP_EOL → Constant kind
    fn walk_in_constant_access(
        &mut self,
        constant_access: &'ast ConstantAccess<'arena>,
        _: &mut (),
    ) {
        self.add_from_identifier(&constant_access.name, NameKind::Constant);
    }
}

#[derive(Debug, Clone)]
pub struct GlobalNamespaceImportRule {
    meta: &'static RuleMeta,
    cfg: GlobalNamespaceImportConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct GlobalNamespaceImportConfig {
    pub level: Level,
    pub import_classes: bool,
    pub import_functions: bool,
    pub import_constants: bool,
}

impl Default for GlobalNamespaceImportConfig {
    fn default() -> Self {
        Self { level: Level::Warning, import_classes: true, import_functions: false, import_constants: false }
    }
}

impl Config for GlobalNamespaceImportConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for GlobalNamespaceImportRule {
    type Config = GlobalNamespaceImportConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Global Namespace Import",
            code: "global-namespace-import",
            description: indoc! {"
                Detects fully-qualified global namespace references (e.g. `\\DateTime`) in namespaced files
                and suggests importing them with `use` statements instead. This improves readability and
                consistency with the rest of the codebase.
            "},
            good_example: indoc! {r"
                <?php

                namespace App;

                use DateTime;

                $d = new DateTime();
            "},
            bad_example: indoc! {r"
                <?php

                namespace App;

                $d = new \DateTime();
            "},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Program];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Program(program) = node else {
            return;
        };

        // At least one import type must be enabled
        if !self.cfg.import_classes && !self.cfg.import_functions && !self.cfg.import_constants {
            return;
        }

        // Check if file has a namespace declaration and collect use statement info
        let mut has_namespace = false;
        let mut use_statement_ranges: Vec<(u32, u32)> = Vec::new();
        let mut last_use_end: Option<u32> = None;
        let mut namespace_end: Option<u32> = None;
        let mut already_imported: HashSet<String> = HashSet::new();

        for stmt in &program.statements {
            match stmt {
                Statement::Namespace(ns) => {
                    has_namespace = true;
                    // For implicit namespace (no braces), the namespace keyword line ends after the name
                    if let Some(name) = &ns.name {
                        namespace_end = Some(name.span().end.offset);
                    }
                    // Check statements inside namespace for use declarations
                    for ns_stmt in ns.statements() {
                        if let Statement::Use(use_stmt) = ns_stmt {
                            let span = ns_stmt.span();
                            use_statement_ranges.push((span.start.offset, span.end.offset));
                            last_use_end = Some(span.end.offset);
                            collect_imported_names(use_stmt, &mut already_imported);
                        }
                    }
                }
                Statement::Use(use_stmt) => {
                    let span = stmt.span();
                    use_statement_ranges.push((span.start.offset, span.end.offset));
                    last_use_end = Some(span.end.offset);
                    collect_imported_names(use_stmt, &mut already_imported);
                }
                _ => {}
            }
        }

        if !has_namespace {
            return;
        }

        // Walk the AST to collect FullyQualified identifiers with their context kind
        let mut collector = FqnCollector::new();
        walk_program_mut(&mut collector, program, &mut ());

        // Group by resolved FQN to handle duplicates, filtering by config and already-imported
        let mut fqcn_occurrences: BTreeMap<String, Vec<Span>> = BTreeMap::new();

        for fqn_ref in &collector.refs {
            // Check if the relevant import type is enabled for this kind
            let enabled = match fqn_ref.kind {
                NameKind::Class => self.cfg.import_classes,
                NameKind::Function => self.cfg.import_functions,
                NameKind::Constant => self.cfg.import_constants,
            };
            if !enabled {
                continue;
            }

            // Skip positions inside use statements
            let offset = fqn_ref.span.start.offset;
            let in_use_stmt =
                use_statement_ranges.iter().any(|(start, end)| offset >= *start && offset < *end);
            if in_use_stmt {
                continue;
            }

            // Skip already imported names
            if already_imported.contains(&fqn_ref.name) {
                continue;
            }

            fqcn_occurrences.entry(fqn_ref.name.clone()).or_default().push(fqn_ref.span);
        }

        // Report each unique FQCN
        let insertion_offset = last_use_end.or(namespace_end);

        for (fqn, occurrences) in &fqcn_occurrences {
            for fqcn_span in occurrences {
                let issue = Issue::new(
                    self.cfg.level(),
                    format!("Fully-qualified name `\\{fqn}` should be imported"),
                )
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(*fqcn_span)
                        .with_message(format!("Use `{fqn}` instead of `\\{fqn}`")),
                )
                .with_help(format!("Add `use {fqn};` and replace `\\{fqn}` with `{fqn}`"));

                if let Some(insert_offset) = insertion_offset {
                    ctx.collector.propose(issue, |edits| {
                        // Replace \ClassName with ClassName
                        edits.push(TextEdit::replace(*fqcn_span, fqn.as_str()));

                        // Add use statement after last use or namespace declaration
                        let insert_pos = Position::new(insert_offset);
                        let insert_span = Span::new(program.file_id, insert_pos, insert_pos);
                        let use_statement = format!("\nuse {fqn};");
                        edits.push(TextEdit::replace(insert_span, &use_statement));
                    });
                } else {
                    ctx.collector.report(issue);
                }
            }
        }
    }
}

fn collect_imported_names(use_stmt: &mago_syntax::ast::Use<'_>, imported: &mut HashSet<String>) {
    use mago_syntax::ast::UseItems;

    match &use_stmt.items {
        UseItems::Sequence(s) => {
            for item in &s.items.nodes {
                let name = item.name.value().trim_start_matches('\\');
                if let Some(last) = name.rsplit('\\').next() {
                    imported.insert(last.to_string());
                } else {
                    imported.insert(name.to_string());
                }
            }
        }
        UseItems::TypedSequence(s) => {
            for item in &s.items.nodes {
                let name = item.name.value().trim_start_matches('\\');
                if let Some(last) = name.rsplit('\\').next() {
                    imported.insert(last.to_string());
                } else {
                    imported.insert(name.to_string());
                }
            }
        }
        UseItems::MixedList(list) => {
            for i in &list.items.nodes {
                let name = i.item.name.value().trim_start_matches('\\');
                if let Some(last) = name.rsplit('\\').next() {
                    imported.insert(last.to_string());
                } else {
                    imported.insert(name.to_string());
                }
            }
        }
        UseItems::TypedList(list) => {
            for item in &list.items.nodes {
                let name = item.name.value().trim_start_matches('\\');
                if let Some(last) = name.rsplit('\\').next() {
                    imported.insert(last.to_string());
                } else {
                    imported.insert(name.to_string());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::GlobalNamespaceImportRule;
    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_failure! {
        name = fqcn_in_namespaced_file_is_bad,
        rule = GlobalNamespaceImportRule,
        settings = |settings: &mut Settings| {
            settings.rules.global_namespace_import.config.import_classes = true;
        },
        code = indoc! {r"
            <?php

            namespace App;

            $d = new \DateTime();
        "}
    }

    test_lint_success! {
        name = already_imported_class_is_ok,
        rule = GlobalNamespaceImportRule,
        settings = |settings: &mut Settings| {
            settings.rules.global_namespace_import.config.import_classes = true;
        },
        code = indoc! {r"
            <?php

            namespace App;

            use DateTime;

            $d = new DateTime();
        "}
    }

    test_lint_success! {
        name = root_namespace_fqcn_is_ok,
        rule = GlobalNamespaceImportRule,
        settings = |settings: &mut Settings| {
            settings.rules.global_namespace_import.config.import_classes = true;
        },
        code = indoc! {r"
            <?php

            $d = new \DateTime();
        "}
    }

    test_lint_success! {
        name = import_classes_disabled_is_ok,
        rule = GlobalNamespaceImportRule,
        settings = |settings: &mut Settings| {
            settings.rules.global_namespace_import.config.import_classes = false;
        },
        code = indoc! {r"
            <?php

            namespace App;

            $d = new \DateTime();
        "}
    }

    test_lint_failure! {
        name = multiple_fqcn_references_reported,
        rule = GlobalNamespaceImportRule,
        count = 2,
        settings = |settings: &mut Settings| {
            settings.rules.global_namespace_import.config.import_classes = true;
        },
        code = indoc! {r"
            <?php

            namespace App;

            $d = new \DateTime();
            $e = new \Exception('test');
        "}
    }

    test_lint_success! {
        name = non_global_fqcn_is_ok,
        rule = GlobalNamespaceImportRule,
        settings = |settings: &mut Settings| {
            settings.rules.global_namespace_import.config.import_classes = true;
        },
        code = indoc! {r"
            <?php

            namespace App;

            use Other\Service\Foo;

            $f = new Foo();
        "}
    }

    // Tests for function/constant kind discrimination

    test_lint_success! {
        name = fqcn_function_not_reported_when_import_functions_disabled,
        rule = GlobalNamespaceImportRule,
        settings = |settings: &mut Settings| {
            settings.rules.global_namespace_import.config.import_classes = true;
            settings.rules.global_namespace_import.config.import_functions = false;
        },
        code = indoc! {r"
            <?php

            namespace App;

            $len = \strlen('hello');
            $found = \in_array(1, [1, 2, 3]);
        "}
    }

    test_lint_failure! {
        name = fqcn_function_reported_when_import_functions_enabled,
        rule = GlobalNamespaceImportRule,
        count = 2,
        settings = |settings: &mut Settings| {
            settings.rules.global_namespace_import.config.import_classes = false;
            settings.rules.global_namespace_import.config.import_functions = true;
        },
        code = indoc! {r"
            <?php

            namespace App;

            $len = \strlen('hello');
            $found = \in_array(1, [1, 2, 3]);
        "}
    }

    test_lint_success! {
        name = fqcn_constant_not_reported_when_import_constants_disabled,
        rule = GlobalNamespaceImportRule,
        settings = |settings: &mut Settings| {
            settings.rules.global_namespace_import.config.import_classes = true;
            settings.rules.global_namespace_import.config.import_constants = false;
        },
        code = indoc! {r"
            <?php

            namespace App;

            $eol = \PHP_EOL;
        "}
    }

    test_lint_failure! {
        name = fqcn_constant_reported_when_import_constants_enabled,
        rule = GlobalNamespaceImportRule,
        settings = |settings: &mut Settings| {
            settings.rules.global_namespace_import.config.import_classes = false;
            settings.rules.global_namespace_import.config.import_constants = true;
        },
        code = indoc! {r"
            <?php

            namespace App;

            $eol = \PHP_EOL;
        "}
    }

    test_lint_failure! {
        name = class_type_hint_reported,
        rule = GlobalNamespaceImportRule,
        settings = |settings: &mut Settings| {
            settings.rules.global_namespace_import.config.import_classes = true;
        },
        code = indoc! {r"
            <?php

            namespace App;

            function foo(\DateTime $d): void {}
        "}
    }

    test_lint_failure! {
        name = class_instanceof_reported,
        rule = GlobalNamespaceImportRule,
        settings = |settings: &mut Settings| {
            settings.rules.global_namespace_import.config.import_classes = true;
        },
        code = indoc! {r"
            <?php

            namespace App;

            $x = $obj instanceof \DateTime;
        "}
    }

    test_lint_failure! {
        name = class_static_method_reported,
        rule = GlobalNamespaceImportRule,
        settings = |settings: &mut Settings| {
            settings.rules.global_namespace_import.config.import_classes = true;
        },
        code = indoc! {r"
            <?php

            namespace App;

            $d = \DateTime::createFromFormat('Y-m-d', '2024-01-01');
        "}
    }

    test_lint_success! {
        name = mixed_kinds_only_classes_reported,
        rule = GlobalNamespaceImportRule,
        settings = |settings: &mut Settings| {
            settings.rules.global_namespace_import.config.import_classes = true;
            settings.rules.global_namespace_import.config.import_functions = false;
            settings.rules.global_namespace_import.config.import_constants = false;
        },
        code = indoc! {r"
            <?php

            namespace App;

            $len = \strlen('hello');
            $eol = \PHP_EOL;
        "}
    }
}
