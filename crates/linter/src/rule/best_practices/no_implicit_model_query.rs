use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoImplicitModelQueryRule {
    meta: &'static RuleMeta,
    cfg: NoImplicitModelQueryConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoImplicitModelQueryConfig {
    pub level: Level,
}

impl Default for NoImplicitModelQueryConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for NoImplicitModelQueryConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoImplicitModelQueryRule {
    type Config = NoImplicitModelQueryConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Implicit Model Query",
            code: "no-implicit-model-query",
            description: indoc! {"
                Detects query-builder methods (`where`, `orderBy`, `with`, ...) called statically on an
                Eloquent model. Such calls are forwarded to a new query through `__callStatic`. Starting
                the chain with an explicit `query()` makes the query obvious and improves IDE support.
            "},
            good_example: indoc! {r"
                <?php

                $users = User::query()->where('is_active', true)->get();
            "},
            bad_example: indoc! {r"
                <?php

                $users = User::where('is_active', true)->get();
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::Integration(Integration::Laravel),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::StaticMethodCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::StaticMethodCall(call) = node else {
            return;
        };

        let Expression::Identifier(class_identifier) = call.class else {
            return;
        };

        let ClassLikeMemberSelector::Identifier(method) = &call.method else {
            return;
        };

        if !is_query_builder_method(method.value) {
            return;
        }

        // A model is never one of Laravel's own facade or helper classes.
        let class_name = ctx.lookup_name(class_identifier);
        if is_known_non_model(last_segment(class_name)) {
            return;
        }

        let method_name = method.value;

        let issue = Issue::new(
            self.cfg.level(),
            "Query-builder methods on a model should be reached through an explicit `query()` call.",
        )
        .with_code(self.meta.code)
        .with_annotation(Annotation::primary(call.span()).with_message(format!(
            "`{}()` is forwarded to a new query implicitly",
            String::from_utf8_lossy(method_name)
        )))
        .with_note(
            "Calling a query-builder method statically on a model is forwarded to a new query through `__callStatic`.",
        )
        .with_help("Start the chain with `query()`, e.g. `Model::query()->where(...)`.");

        ctx.collector.propose(issue, |edits| {
            edits.push(TextEdit::insert(method.span().start.offset, "query()->"));
        });
    }
}

/// Returns the final `\`-separated segment of a class name.
fn last_segment(name: &[u8]) -> &[u8] {
    match name.iter().rposition(|&byte| byte == b'\\') {
        Some(position) => &name[position + 1..],
        None => name,
    }
}

fn is_query_builder_method(name: &[u8]) -> bool {
    QUERY_BUILDER_METHODS.iter().any(|method| name.eq_ignore_ascii_case(method))
}

fn is_known_non_model(name: &[u8]) -> bool {
    KNOWN_NON_MODEL_CLASSES.iter().any(|class| name.eq_ignore_ascii_case(class))
}

/// Eloquent query-builder methods that start or constrain a query when forwarded from a model.
const QUERY_BUILDER_METHODS: &[&[u8]] = &[
    b"where",
    b"orWhere",
    b"whereNot",
    b"whereIn",
    b"whereNotIn",
    b"whereNull",
    b"whereNotNull",
    b"whereBetween",
    b"whereNotBetween",
    b"whereDate",
    b"whereTime",
    b"whereColumn",
    b"whereExists",
    b"whereHas",
    b"whereDoesntHave",
    b"whereRelation",
    b"whereBelongsTo",
    b"orderBy",
    b"orderByDesc",
    b"latest",
    b"oldest",
    b"inRandomOrder",
    b"groupBy",
    b"having",
    b"with",
    b"withCount",
    b"withSum",
    b"withMax",
    b"withMin",
    b"withAvg",
    b"withWhereHas",
    b"withTrashed",
    b"withoutTrashed",
    b"onlyTrashed",
    b"has",
    b"orHas",
    b"doesntHave",
    b"limit",
    b"take",
    b"skip",
    b"offset",
    b"distinct",
];

/// Laravel facade and helper classes that own static methods sharing query-builder names.
/// A model never carries one of these short names, so they are excluded from the rule.
const KNOWN_NON_MODEL_CLASSES: &[&[u8]] = &[
    b"Arr",
    b"Str",
    b"Number",
    b"Js",
    b"Collection",
    b"LazyCollection",
    b"Fluent",
    b"Stringable",
    b"Uri",
    b"DB",
    b"Schema",
    b"Cache",
    b"Config",
    b"Route",
    b"View",
    b"Log",
    b"Http",
    b"Storage",
    b"File",
    b"Validator",
    b"Hash",
    b"Auth",
    b"Gate",
    b"Event",
    b"Queue",
    b"Mail",
    b"Notification",
    b"Session",
    b"Cookie",
    b"Redirect",
    b"Response",
    b"Request",
    b"URL",
    b"App",
    b"Artisan",
    b"Blade",
    b"Bus",
    b"Date",
    b"Lang",
    b"Password",
    b"Process",
    b"RateLimiter",
    b"Pipeline",
    b"Context",
    b"Crypt",
    b"Broadcast",
    b"Concurrency",
    b"Vite",
];

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_failure! {
        name = static_where_on_model,
        rule = NoImplicitModelQueryRule,
        code = indoc! {r"
            <?php

            $users = User::where('is_active', true)->get();
        "}
    }

    test_lint_failure! {
        name = static_with_on_model,
        rule = NoImplicitModelQueryRule,
        count = 1,
        code = indoc! {r"
            <?php

            $posts = Post::with('author')->latest()->get();
        "}
    }

    test_lint_success! {
        name = explicit_query,
        rule = NoImplicitModelQueryRule,
        code = indoc! {r"
            <?php

            $users = User::query()->where('is_active', true)->get();
        "}
    }

    test_lint_success! {
        name = finder_methods_allowed,
        rule = NoImplicitModelQueryRule,
        code = indoc! {r"
            <?php

            $user = User::find(1);
            $first = User::create(['name' => 'Jane']);
        "}
    }

    test_lint_success! {
        name = support_helper_not_flagged,
        rule = NoImplicitModelQueryRule,
        code = indoc! {r"
            <?php

            $filtered = Arr::where($items, fn ($value) => $value !== null);
        "}
    }

    test_lint_fix! {
        name = fix_inserts_query,
        rule = NoImplicitModelQueryRule,
        code = indoc! {r"
            <?php

            $users = User::where('is_active', true)->get();
        "},
        fixed = indoc! {r"
            <?php

            $users = User::query()->where('is_active', true)->get();
        "}
    }
}
