//! Native parameter hints from literals and the exact variable use at the call.

use mago_analyzer::artifacts::editor::EditorFileData;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::{TAtomic, object::TObject, scalar::TScalar};
use mago_codex::ttype::union::TUnion;
use mago_php_version::PHPVersion;
use mago_span::HasSpan;
use mago_syntax::cst::{Expression, Literal, UnaryPrefixOperator, Variable};

use crate::Server;

pub(super) fn hint(server: &Server, editor: Option<&EditorFileData>, expression: &Expression<'_>) -> Option<String> {
    let expression = expression.unparenthesized();
    match expression {
        Expression::Variable(Variable::Direct(variable)) => {
            let (span, union) = editor?.variable_types.at_offset(variable.start_offset())?;
            if span != (variable.start_offset(), variable.end_offset()) {
                return None;
            }
            native_union(server, union)
        }
        Expression::Literal(literal) => literal_hint(server, literal),
        Expression::UnaryPrefix(unary)
            if matches!(unary.operator, UnaryPrefixOperator::Plus(_) | UnaryPrefixOperator::Negation(_)) =>
        {
            match unary.operand.unparenthesized() {
                Expression::Literal(literal @ (Literal::Integer(_) | Literal::Float(_))) => {
                    literal_hint(server, literal)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn literal_hint(server: &Server, literal: &Literal<'_>) -> Option<String> {
    if server.php_version < PHPVersion::PHP70 || matches!(literal, Literal::Null(_)) {
        return None;
    }
    super::super::refactor_literal::native_hint(literal).map(str::to_owned)
}

fn native_union(server: &Server, union: &TUnion) -> Option<String> {
    if union.possibly_undefined() || union.possibly_undefined_from_try() {
        return None;
    }
    let mut hints = Vec::new();
    let mut nullable = false;
    for atomic in union.types.iter() {
        if atomic.has_intersection_types() {
            return None;
        }
        let hint = match atomic {
            TAtomic::Scalar(scalar) if server.php_version >= PHPVersion::PHP70 => match scalar {
                TScalar::Bool(_) => "bool",
                TScalar::Integer(_) => "int",
                TScalar::Float(_) => "float",
                TScalar::String(_) | TScalar::ClassLikeString(_) => "string",
                _ => return None,
            },
            TAtomic::Object(TObject::Any) if server.php_version >= PHPVersion::PHP72 => "object",
            TAtomic::Object(object @ (TObject::Named(_) | TObject::Enum(_))) => {
                if matches!(object, TObject::Named(named) if named.has_type_parameters()) {
                    return None;
                }
                let class = server.codebase().get_class_like_by_name(object.get_name()?)?;
                if class.kind.is_trait() || class.name_span.is_none() {
                    return None;
                }
                hints.push(format!("\\{}", std::str::from_utf8(class.original_name.as_bytes()).ok()?));
                continue;
            }
            TAtomic::Null => {
                nullable = true;
                continue;
            }
            _ => return None,
        };
        hints.push(hint.to_owned());
    }
    hints.sort_unstable();
    hints.dedup();
    if hints.is_empty()
        || (hints.iter().any(|hint| hint == "object") && hints.iter().any(|hint| hint.starts_with('\\')))
    {
        return None;
    }
    if hints.len() == 1 {
        if nullable {
            return (server.php_version >= PHPVersion::PHP71).then(|| format!("?{}", hints[0]));
        }
        return hints.pop();
    }
    if server.php_version < PHPVersion::PHP80 {
        return None;
    }
    if nullable {
        hints.push("null".to_owned());
    }
    Some(hints.join("|"))
}
