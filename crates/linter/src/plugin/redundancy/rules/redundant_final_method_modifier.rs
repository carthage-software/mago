use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RedundantFinalMethodModifierRule;

impl RedundantFinalMethodModifierRule {
    fn report(&self, method: &Method, context: &mut LintContext<'_>, in_enum: bool) {
        let Some(final_modifier) = method.modifiers.get_final() else {
            return;
        };

        let method_name = context.interner.lookup(&method.name.value);

        let message = if in_enum {
            format!("The `final` modifier on enum method `{}` is redundant as enums cannot be extended.", method_name,)
        } else {
            format!("The `final` modifier on method `{}` is redundant as the class is already final.", method_name,)
        };

        let issue = Issue::new(context.level(), message)
            .with_annotations([
                Annotation::primary(final_modifier.span()).with_message("This `final` modifier is redundant.")
            ])
            .with_help("Remove the redundant `final` modifier.");

        context
            .report_with_fix(issue, |plan| plan.delete(final_modifier.span().to_range(), SafetyClassification::Safe));
    }
}

impl Rule for RedundantFinalMethodModifierRule {
    fn get_name(&self) -> &'static str {
        "redundant-final-method-modifier"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Help)
    }
}

impl<'a> Walker<LintContext<'a>> for RedundantFinalMethodModifierRule {
    fn walk_in_class<'ast>(&self, class: &'ast Class, context: &mut LintContext<'a>) {
        if !class.modifiers.contains_final() {
            return;
        }

        for member in class.members.iter() {
            if let ClassLikeMember::Method(method) = member {
                self.report(method, context, false);
            }
        }
    }

    fn walk_in_enum<'ast>(&self, r#enum: &'ast Enum, context: &mut LintContext<'a>) {
        for member in r#enum.members.iter() {
            if let ClassLikeMember::Method(method) = member {
                self.report(method, context, true);
            }
        }
    }
}
