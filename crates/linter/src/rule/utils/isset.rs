use mago_syntax::ast::Access;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Literal;
use mago_syntax::ast::Variable;

/// A single step in an access path, representing either the root variable
/// or one level of array/property access.
#[derive(Debug, PartialEq, Eq)]
pub enum AccessStep<'ast> {
    DirectVariable(&'ast str),
    ArrayStringIndex(&'ast str),
    ArrayIntegerIndex(u64),
    Property(&'ast str),
    NullSafeProperty(&'ast str),
}

/// The full access path of an expression, from root to leaf.
///
/// For `$foo->bar['baz']`, this would be:
/// `[DirectVariable("$foo"), Property("bar"), ArrayStringIndex("baz")]`
pub type AccessPath<'ast> = Vec<AccessStep<'ast>>;

/// Builds a structural access path for an expression.
pub fn build_access_path<'arena>(expr: &Expression<'arena>) -> Option<AccessPath<'arena>> {
    let mut steps = Vec::new();
    let mut current = expr;

    loop {
        match current {
            Expression::ArrayAccess(access) => {
                match access.index {
                    Expression::Literal(Literal::String(s)) => {
                        steps.push(AccessStep::ArrayStringIndex(s.value.unwrap_or(s.raw)));
                    }
                    Expression::Literal(Literal::Integer(i)) => {
                        steps.push(AccessStep::ArrayIntegerIndex(i.value?));
                    }
                    _ => return None,
                }

                current = access.array;
            }
            Expression::Access(Access::Property(prop)) => {
                let ClassLikeMemberSelector::Identifier(ident) = &prop.property else {
                    return None;
                };

                steps.push(AccessStep::Property(ident.value));
                current = prop.object;
            }
            Expression::Access(Access::NullSafeProperty(prop)) => {
                let ClassLikeMemberSelector::Identifier(ident) = &prop.property else {
                    return None;
                };

                steps.push(AccessStep::NullSafeProperty(ident.value));
                current = prop.object;
            }
            Expression::Variable(Variable::Direct(var)) => {
                steps.push(AccessStep::DirectVariable(var.name));
                break;
            }
            _ => return None,
        }
    }

    steps.reverse();
    Some(steps)
}

/// Returns `true` if `a` is a proper prefix of `b`.
pub fn is_proper_prefix(a: &[AccessStep<'_>], b: &[AccessStep<'_>]) -> bool {
    a.len() < b.len() && a == &b[..a.len()]
}

/// Returns `true` if `a` is redundant given `b` (i.e., `a` is a proper prefix of `b` or equal to `b`).
pub fn is_redundant(a: &[AccessStep<'_>], b: &[AccessStep<'_>]) -> bool {
    a.len() <= b.len() && a == &b[..a.len()]
}
