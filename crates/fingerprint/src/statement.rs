use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Statement<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self {
            Statement::OpeningTag(tag) => tag.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::ClosingTag(tag) => tag.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Inline(inline) => inline.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Namespace(namespace) => namespace.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Use(r#use) => r#use.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Class(class) => class.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Interface(interface) => interface.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Trait(r#trait) => r#trait.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Enum(r#enum) => r#enum.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Block(block) => block.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Constant(constant) => constant.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Function(function) => function.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Declare(declare) => declare.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Goto(goto) => goto.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Label(label) => label.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Try(r#try) => r#try.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Foreach(foreach) => foreach.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::For(r#for) => r#for.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::While(r#while) => r#while.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::DoWhile(do_while) => do_while.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Continue(r#continue) => r#continue.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Break(r#break) => r#break.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Switch(switch) => switch.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::If(r#if) => r#if.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Return(r#return) => r#return.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Expression(expr_stmt) => expr_stmt.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Echo(echo) => echo.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::EchoTag(echo_tag) => echo_tag.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Global(global) => global.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Static(r#static) => r#static.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::HaltCompiler(halt) => halt.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Unset(unset) => unset.fingerprint_with_hasher(hasher, resolved_names, options),
            Statement::Noop(_) => {
                // Noop statements do not contribute to the fingerprint
            }
        }
    }
}

impl Fingerprintable for ExpressionStatement<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        self.expression.fingerprint_with_hasher(hasher, resolved_names, options);
        self.terminator.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
