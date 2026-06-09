use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Class;
use mago_syntax::ast::Enum;
use mago_syntax::ast::Function;
use mago_syntax::ast::Interface;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Trait;
use mago_syntax::walker::MutWalker;
use mago_syntax::walker::walk_program_mut;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;
use mago_bytes::BytesDisplay;

#[derive(Debug, Clone)]
pub struct FileNameRule {
    meta: &'static RuleMeta,
    cfg: FileNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct FileNameConfig {
    pub level: Level,
    pub check_functions: bool,
}

impl Default for FileNameConfig {
    fn default() -> Self {
        Self { level: Level::Warning, check_functions: false }
    }
}

impl Config for FileNameConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for FileNameRule {
    type Config = FileNameConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "File Name",
            code: "file-name",
            description: indoc! {"
                Ensures that a file containing a single class-like definition is named after that definition.

                For example, a file containing `class Foo` must be named `Foo.php`.
                Optionally, this rule can also check functions: a file containing a single function `foo`
                must be named `foo.php`.
            "},
            good_example: indoc! {r#"
                <?php
                // File: test.php

                namespace App;

                class test
                {
                }
            "#},
            bad_example: indoc! {r#"
                <?php
                // File: test.php

                namespace App;

                class Foo
                {
                }
            "#},
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

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Program(program) = node else {
            return;
        };

        let mut collector = DefinitionCollector { class_likes: Vec::new(), functions: Vec::new() };
        walk_program_mut(&mut collector, program, &mut ());

        let file_name_bytes: &[u8] = ctx.source_file.name.as_ref();
        let file_stem_bytes = match memchr::memrchr(b'/', file_name_bytes) {
            Some(pos) => &file_name_bytes[pos + 1..],
            None => file_name_bytes,
        };
        let file_stem_bytes = if let Some(stripped) = file_stem_bytes.strip_suffix(b".php") {
            stripped
        } else {
            match memchr::memrchr(b'.', file_stem_bytes) {
                Some(pos) => &file_stem_bytes[..pos],
                None => file_stem_bytes,
            }
        };
        let file_stem = BytesDisplay(file_stem_bytes);

        if collector.class_likes.len() == 1 {
            let (kind, name_bytes, span) = collector.class_likes[0];
            let name = BytesDisplay(name_bytes);

            if name_bytes != file_stem_bytes {
                let issue = Issue::new(
                    self.cfg.level(),
                    format!("{kind} `{name}` should be in a file named `{name}.php`, found `{file_stem}.php`."),
                )
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(span).with_message(format!("{kind} `{name}` defined here")))
                .with_help(format!(
                    "Rename the file to `{name}.php` or rename the {} to match the file name.",
                    kind.to_lowercase()
                ));

                ctx.collector.report(issue);
            }

            return;
        }

        if self.cfg.check_functions && collector.class_likes.is_empty() && collector.functions.len() == 1 {
            let (name_bytes, span) = collector.functions[0];
            let name = BytesDisplay(name_bytes);

            if name_bytes != file_stem_bytes {
                let issue = Issue::new(
                    self.cfg.level(),
                    format!("Function `{name}` should be in a file named `{name}.php`, found `{file_stem}.php`."),
                )
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(span).with_message(format!("Function `{name}` defined here")))
                .with_help(format!("Rename the file to `{name}.php` or rename the function to match the file name."));

                ctx.collector.report(issue);
            }
        }
    }
}

struct DefinitionCollector<'ast> {
    class_likes: Vec<(&'static str, &'ast [u8], Span)>,
    functions: Vec<(&'ast [u8], Span)>,
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for DefinitionCollector<'ast> {
    fn walk_in_class(&mut self, class: &'ast Class<'arena>, _: &mut ()) {
        self.class_likes.push(("Class", class.name.value, class.name.span()));
    }

    fn walk_in_interface(&mut self, interface: &'ast Interface<'arena>, _: &mut ()) {
        self.class_likes.push(("Interface", interface.name.value, interface.name.span()));
    }

    fn walk_in_trait(&mut self, r#trait: &'ast Trait<'arena>, _: &mut ()) {
        self.class_likes.push(("Trait", r#trait.name.value, r#trait.name.span()));
    }

    fn walk_in_enum(&mut self, r#enum: &'ast Enum<'arena>, _: &mut ()) {
        self.class_likes.push(("Enum", r#enum.name.value, r#enum.name.span()));
    }

    fn walk_in_function(&mut self, function: &'ast Function<'arena>, _: &mut ()) {
        self.functions.push((function.name.value, function.name.span()));
    }
}
