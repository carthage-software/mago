use mago_atom::Atom;
use mago_atom::ascii_lowercase_atom;
use mago_atom::ascii_lowercase_constant_name_atom;
use mago_syntax::ast::ArgumentList;
use mago_syntax::ast::Binary;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::Call;
use mago_syntax::ast::Expression;
use mago_syntax::ast::FunctionCall;
use mago_syntax::ast::Parenthesized;
use mago_syntax::ast::UnaryPrefix;
use mago_syntax::ast::UnaryPrefixOperator;

use crate::artifacts::AnalysisArtifacts;
use crate::context::block::BlockContext;

/// Extracts `function_exists` and `defined` calls from a condition expression.
pub fn extract_function_constant_existence(
    expression: &Expression<'_>,
    artifacts: &AnalysisArtifacts,
    block_context: &mut BlockContext<'_>,
    negated: bool,
) {
    match expression {
        Expression::UnaryPrefix(UnaryPrefix { operator: UnaryPrefixOperator::Not(_), operand }) => {
            extract_function_constant_existence(operand, artifacts, block_context, !negated);
        }
        Expression::Binary(Binary { lhs, operator: BinaryOperator::And(_) | BinaryOperator::LowAnd(_), rhs })
            if !negated =>
        {
            extract_function_constant_existence(lhs, artifacts, block_context, negated);
            extract_function_constant_existence(rhs, artifacts, block_context, negated);
        }
        Expression::Binary(Binary { lhs, operator: BinaryOperator::Or(_) | BinaryOperator::LowOr(_), rhs })
            if negated =>
        {
            extract_function_constant_existence(lhs, artifacts, block_context, negated);
            extract_function_constant_existence(rhs, artifacts, block_context, negated);
        }
        Expression::Call(Call::Function(FunctionCall {
            function: Expression::Identifier(ident),
            argument_list,
            ..
        })) if !negated => {
            let func_name = ident.value().to_ascii_lowercase();
            match func_name.as_str() {
                "function_exists" => {
                    if let Some(name) = get_first_literal_string_arg(argument_list, artifacts, false) {
                        block_context.known_functions.insert(name);
                    }
                }
                "defined" => {
                    if let Some(name) = get_first_literal_string_arg(argument_list, artifacts, true) {
                        block_context.known_constants.insert(name);
                    }
                }
                _ => {}
            }
        }
        Expression::Parenthesized(Parenthesized { expression, .. }) => {
            extract_function_constant_existence(expression, artifacts, block_context, negated);
        }
        _ => {}
    }
}

fn get_first_literal_string_arg(
    argument_list: &ArgumentList,
    artifacts: &AnalysisArtifacts,
    constant: bool,
) -> Option<Atom> {
    argument_list
        .arguments
        .first()
        .map(mago_syntax::ast::Argument::value)
        .and_then(|expr| artifacts.get_expression_type(expr))
        .and_then(|ty| ty.get_single_literal_string_value())
        .map(|s| if constant { ascii_lowercase_constant_name_atom(s) } else { ascii_lowercase_atom(s) })
}
