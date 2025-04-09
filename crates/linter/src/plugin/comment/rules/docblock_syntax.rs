use indoc::indoc;

use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::directive::LintDirective;
use crate::rule::Rule;

/// TODO(azjezz): Enable this rule by default once we have improved the linting experience.
#[derive(Clone, Debug)]
pub struct DocblockSyntaxRule;

impl Rule for DocblockSyntaxRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::disabled("Docblock Syntax").with_description(indoc! {"
            Checks for syntax errors in docblock comments. This rule is disabled by default because
            it can be noisy and may not be relevant to all codebases.
        "})
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Program(program) = node else { return LintDirective::Abort };

        for trivia in program.trivia.iter() {
            if let TriviaKind::DocBlockComment = trivia.kind {
                let Err(parse_error) = mago_docblock::parse_trivia(context.interner, trivia) else {
                    continue;
                };

                let issue = Issue::new(context.level(), parse_error.to_string())
                    .with_annotation(Annotation::primary(parse_error.span()))
                    .with_annotation(Annotation::secondary(trivia.span()))
                    .with_note(parse_error.note())
                    .with_help(parse_error.help());

                context.report(issue);
            }
        }

        LintDirective::Abort
    }
}
