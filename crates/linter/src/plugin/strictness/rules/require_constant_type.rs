use indoc::indoc;

use mago_ast::ast::*;
use mago_reporting::*;
use mago_span::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RequireConstantTypeRule;

impl Rule for RequireConstantTypeRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Require Constant Type", Level::Warning)
            .with_description(indoc! {"
                Detects class constants that are missing a type hint.
            "})
            .with_example(RuleUsageExample::valid(
                "A class constant with a type hint",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    namespace Psl\IO\Internal;

                    use Psl\IO;

                    class ResourceHandle implements IO\CloseSeekReadWriteStreamHandleInterface
                    {
                        use IO\ReadHandleConvenienceMethodsTrait;
                        use IO\WriteHandleConvenienceMethodsTrait;

                        public const int DEFAULT_READ_BUFFER_SIZE = 4096;
                        public const int MAXIMUM_READ_BUFFER_SIZE = 786432;

                        // ...
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A class constant without a type hint",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    namespace Psl\IO\Internal;

                    use Psl\IO;

                    class ResourceHandle implements IO\CloseSeekReadWriteStreamHandleInterface
                    {
                        use IO\ReadHandleConvenienceMethodsTrait;
                        use IO\WriteHandleConvenienceMethodsTrait;

                        public const DEFAULT_READ_BUFFER_SIZE = 4096;
                        public const MAXIMUM_READ_BUFFER_SIZE = 786432;

                        // ...
                    }
                "#},
            ))
    }
}

impl<'a> Walker<LintContext<'a>> for RequireConstantTypeRule {
    fn walk_class_like_constant<'ast>(
        &self,
        class_like_constant: &'ast ClassLikeConstant,
        context: &mut LintContext<'a>,
    ) {
        if class_like_constant.hint.is_some() {
            return;
        }

        let item = class_like_constant.first_item();

        let constant_name = context.lookup(&item.name.value);

        context.report(
            Issue::new(context.level(), format!("Class constant `{}` is missing a type hint.", constant_name))
                .with_annotation(
                    Annotation::primary(class_like_constant.span())
                        .with_message(format!("Class constant `{}` is defined here.", constant_name)),
                )
                .with_note("Adding a type hint to constants improves code readability and helps prevent type errors.")
                .with_help(format!("Consider specifying a type hint for `{}`.", constant_name)),
        );
    }
}
