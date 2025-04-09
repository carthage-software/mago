use crate::ast::*;

/// Determine if a statement contains only definitions.
#[inline]
pub fn statement_contains_only_definitions(statement: &Statement) -> bool {
    let (definitions, statements) = get_statement_stats(statement);

    definitions != 0 && statements == 0
}

#[inline]
pub fn statement_sequence_contains_only_definitions(statement: &Sequence<Statement>) -> bool {
    let mut definitions = 0;
    let mut statements = 0;
    for statement in statement.iter() {
        let (def, stmt) = get_statement_stats(statement);

        definitions += def;
        statements += stmt;
    }

    definitions != 0 && statements == 0
}

#[inline]
fn get_statement_stats(statement: &Statement) -> (usize, usize) {
    let mut total_definitions = 0;
    let mut total_statements = 0;

    match &statement {
        Statement::Namespace(namespace) => {
            for statement in namespace.statements().iter() {
                let (definitions, statements) = get_statement_stats(statement);
                total_definitions += definitions;
                total_statements += statements;
            }
        }
        Statement::Block(block) => {
            for statement in block.statements.iter() {
                let (definitions, statements) = get_statement_stats(statement);
                total_definitions += definitions;
                total_statements += statements;
            }
        }
        Statement::Class(_)
        | Statement::Interface(_)
        | Statement::Trait(_)
        | Statement::Enum(_)
        | Statement::Function(_)
        | Statement::Constant(_) => {
            total_definitions += 1;
        }
        _ => {
            total_statements += 1;
        }
    }

    (total_definitions, total_statements)
}
