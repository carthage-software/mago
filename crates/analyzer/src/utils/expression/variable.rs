use ahash::HashSet;

use mago_span::Span;
use mago_syntax::ast::*;
use mago_syntax::walker::*;

#[inline]
pub fn get_variables_referenced_in_expression<'arena>(
    expression: &Expression<'arena>,
    ignore_symbols: bool,
) -> HashSet<(&'arena str, Span)> {
    let mut scanner = VariableScanner { ignore_symbols, ..Default::default() };

    scanner.walk_expression(expression, &mut ());
    scanner.variables
}

#[derive(Debug, Default)]
struct VariableScanner<'arena> {
    ignore_symbols: bool,
    variables: HashSet<(&'arena str, Span)>,
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for VariableScanner<'arena> {
    #[inline]
    fn walk_in_direct_variable(&mut self, direct_variable: &'ast DirectVariable<'arena>, _: &mut ()) {
        self.variables.insert((direct_variable.name, direct_variable.span));
    }

    #[inline]
    fn walk_arrow_function(&mut self, arrow_function: &'ast ArrowFunction<'arena>, context: &mut ()) {
        if self.ignore_symbols {
            self.walk_expression(arrow_function.expression, context);

            return;
        }

        walk_arrow_function_mut(self, arrow_function, context);
    }

    #[inline]
    fn walk_closure(&mut self, closure: &'ast Closure<'arena>, context: &mut ()) {
        if self.ignore_symbols {
            if let Some(closure_use_clause) = closure.use_clause.as_ref() {
                walk_closure_use_clause_mut(self, closure_use_clause, context);
            }

            return;
        }

        walk_closure_mut(self, closure, context);
    }

    #[inline]
    fn walk_function(&mut self, function: &'ast Function<'arena>, context: &mut ()) {
        if self.ignore_symbols {
            return;
        }

        walk_function_mut(self, function, context);
    }

    #[inline]
    fn walk_class(&mut self, class: &'ast Class<'arena>, context: &mut ()) {
        if self.ignore_symbols {
            return;
        }

        walk_class_mut(self, class, context);
    }

    #[inline]
    fn walk_interface(&mut self, interface: &'ast Interface<'arena>, context: &mut ()) {
        if self.ignore_symbols {
            return;
        }

        walk_interface_mut(self, interface, context);
    }

    #[inline]
    fn walk_trait(&mut self, trait_: &'ast Trait<'arena>, context: &mut ()) {
        if self.ignore_symbols {
            return;
        }

        walk_trait_mut(self, trait_, context);
    }

    #[inline]
    fn walk_enum(&mut self, enum_: &'ast Enum<'arena>, context: &mut ()) {
        if self.ignore_symbols {
            return;
        }

        walk_enum_mut(self, enum_, context);
    }
}
