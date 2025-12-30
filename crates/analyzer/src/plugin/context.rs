//! Context types for providers and hooks.

use std::cell::RefCell;
use std::rc::Rc;

use mago_atom::Atom;
use mago_atom::atom;
use mago_codex::context::ScopeContext;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::atomic::scalar::string::TStringLiteral;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Argument;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;
use mago_syntax::ast::PartialApplication;
use mago_syntax::ast::PartialArgument;

use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::block::BlockContext;
use crate::invocation::Invocation;
use crate::invocation::InvocationArgument;
use crate::invocation::InvocationArgumentsSource;

pub struct ReportedIssue {
    pub code: IssueCode,
    pub issue: Issue,
}

pub struct ProviderContext<'a, 'b, 'c> {
    pub(crate) codebase: &'a CodebaseMetadata,
    pub(crate) artifacts: &'b AnalysisArtifacts,
    pub(crate) block_context: &'c BlockContext<'a>,
    pub(crate) reported_issues: RefCell<Vec<ReportedIssue>>,
}

impl<'a, 'b, 'c> ProviderContext<'a, 'b, 'c> {
    pub(crate) fn new(
        codebase: &'a CodebaseMetadata,
        block_context: &'c BlockContext<'a>,
        artifacts: &'b AnalysisArtifacts,
    ) -> Self {
        Self { codebase, artifacts, block_context, reported_issues: RefCell::new(Vec::new()) }
    }

    pub fn report(&self, code: IssueCode, issue: Issue) {
        self.reported_issues.borrow_mut().push(ReportedIssue { code, issue });
    }

    pub(crate) fn take_issues(&self) -> Vec<ReportedIssue> {
        std::mem::take(&mut *self.reported_issues.borrow_mut())
    }

    #[inline]
    pub fn codebase(&self) -> &'a CodebaseMetadata {
        self.codebase
    }

    #[inline]
    pub fn get_expression_type<T: HasSpan>(&self, expr: &T) -> Option<&TUnion> {
        self.artifacts.get_expression_type(expr)
    }

    #[inline]
    pub fn get_rc_expression_type<T: HasSpan>(&self, expr: &T) -> Option<&Rc<TUnion>> {
        self.artifacts.get_rc_expression_type(expr)
    }

    #[inline]
    pub fn get_variable_type(&self, name: &str) -> Option<&Rc<TUnion>> {
        self.block_context.locals.get(&atom(name))
    }

    #[inline]
    pub fn scope(&self) -> &ScopeContext<'a> {
        &self.block_context.scope
    }

    #[inline]
    pub fn is_instance_of(&self, class: &str, parent: &str) -> bool {
        self.codebase.is_instance_of(class, parent)
    }

    #[inline]
    pub fn get_closure_metadata<'arena>(&self, expr: &Expression<'arena>) -> Option<&'a FunctionLikeMetadata> {
        match expr {
            Expression::ArrowFunction(arrow_fn) => {
                let span = arrow_fn.span();
                self.codebase.get_closure(&span.file_id, &span.start)
            }
            Expression::Closure(closure) => {
                let span = closure.span();
                self.codebase.get_closure(&span.file_id, &span.start)
            }
            _ => None,
        }
    }

    /// Get metadata for a callable expression (closure, arrow function, or first-class callable).
    ///
    /// This method extends `get_closure_metadata` to also handle first-class callables
    /// like `is_string(...)` or `SomeClass::method(...)`, as well as string literals representing callables.
    #[inline]
    pub fn get_callable_metadata<'arena>(&self, expr: &Expression<'arena>) -> Option<&'a FunctionLikeMetadata> {
        match expr {
            Expression::ArrowFunction(arrow_fn) => {
                let span = arrow_fn.span();

                self.codebase.get_closure(&span.file_id, &span.start)
            }
            Expression::Closure(closure) => {
                let span = closure.span();

                self.codebase.get_closure(&span.file_id, &span.start)
            }
            Expression::PartialApplication(partial) => match partial {
                PartialApplication::Function(func_partial) => {
                    if !func_partial.argument_list.is_first_class_callable() {
                        return None;
                    }

                    if let Expression::Identifier(identifier) = func_partial.function {
                        self.codebase.get_function(identifier.value())
                    } else {
                        None
                    }
                }
                PartialApplication::StaticMethod(static_partial) => {
                    if !static_partial.argument_list.is_first_class_callable() {
                        return None;
                    }

                    if let Expression::Identifier(class_id) = static_partial.class {
                        if let ClassLikeMemberSelector::Identifier(method_id) = &static_partial.method {
                            self.codebase.get_method(class_id.value(), method_id.value)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                PartialApplication::Method(_) => None,
            },
            _ => {
                let expr_type = self.get_rc_expression_type(expr)?;
                if !expr_type.is_single() {
                    return None;
                }

                match expr_type.get_single() {
                    TAtomic::Callable(first_callable) => {
                        if let Some(identifier) = first_callable.get_alias() {
                            self.codebase.get_function_like(identifier)
                        } else {
                            None
                        }
                    }
                    TAtomic::Scalar(TScalar::String(TString {
                        literal: Some(TStringLiteral::Value(literal_string)),
                        ..
                    })) => {
                        if let Some((class_like, method_name)) = literal_string.split_once("::") {
                            self.codebase.get_method(class_like, method_name)
                        } else {
                            self.codebase.get_function(literal_string)
                        }
                    }
                    _ => None,
                }
            }
        }
    }

    #[inline]
    pub fn get_class_like(&self, name: &Atom) -> Option<&ClassLikeMetadata> {
        self.codebase.get_class_like(name)
    }

    #[inline]
    pub fn current_class_name(&self) -> Option<Atom> {
        self.block_context.scope.get_class_like_name()
    }
}

/// Context for hooks that provides mutable access to analysis state.
///
/// Unlike `ProviderContext` which is read-only, `HookContext` allows hooks
/// to modify the analysis state (expression types, variable types, assertions).
pub struct HookContext<'ctx, 'a> {
    pub(crate) codebase: &'ctx CodebaseMetadata,
    pub(crate) block_context: &'a mut BlockContext<'ctx>,
    pub(crate) artifacts: &'a mut AnalysisArtifacts,
    pub(crate) reported_issues: RefCell<Vec<ReportedIssue>>,
}

impl<'ctx, 'a> HookContext<'ctx, 'a> {
    pub(crate) fn new(
        codebase: &'ctx CodebaseMetadata,
        block_context: &'a mut BlockContext<'ctx>,
        artifacts: &'a mut AnalysisArtifacts,
    ) -> Self {
        Self { codebase, artifacts, block_context, reported_issues: RefCell::new(Vec::new()) }
    }

    /// Report an issue from a hook.
    pub fn report(&self, code: IssueCode, issue: Issue) {
        self.reported_issues.borrow_mut().push(ReportedIssue { code, issue });
    }

    pub(crate) fn take_issues(&self) -> Vec<ReportedIssue> {
        std::mem::take(&mut *self.reported_issues.borrow_mut())
    }

    /// Get access to the codebase metadata.
    #[inline]
    pub fn codebase(&self) -> &'ctx CodebaseMetadata {
        self.codebase
    }

    /// Get the type of an expression.
    #[inline]
    pub fn get_expression_type<T: HasSpan>(&self, expr: &T) -> Option<&TUnion> {
        self.artifacts.get_expression_type(expr)
    }

    /// Get the type of an expression as an Rc.
    #[inline]
    pub fn get_rc_expression_type<T: HasSpan>(&self, expr: &T) -> Option<&Rc<TUnion>> {
        self.artifacts.get_rc_expression_type(expr)
    }

    /// Get the type of a variable.
    #[inline]
    pub fn get_variable_type(&self, name: &str) -> Option<&Rc<TUnion>> {
        self.block_context.locals.get(&atom(name))
    }

    /// Get the current scope context.
    #[inline]
    pub fn scope(&self) -> &ScopeContext<'ctx> {
        &self.block_context.scope
    }

    /// Check if a class is an instance of another class.
    #[inline]
    pub fn is_instance_of(&self, class: &str, parent: &str) -> bool {
        self.codebase.is_instance_of(class, parent)
    }

    /// Get metadata for a closure expression.
    #[inline]
    pub fn get_closure_metadata<'arena>(&self, expr: &Expression<'arena>) -> Option<&'ctx FunctionLikeMetadata> {
        match expr {
            Expression::ArrowFunction(arrow_fn) => {
                let span = arrow_fn.span();
                self.codebase.get_closure(&span.file_id, &span.start)
            }
            Expression::Closure(closure) => {
                let span = closure.span();
                self.codebase.get_closure(&span.file_id, &span.start)
            }
            _ => None,
        }
    }

    /// Get metadata for a class-like by name.
    #[inline]
    pub fn get_class_like(&self, name: &Atom) -> Option<&ClassLikeMetadata> {
        self.codebase.get_class_like(name)
    }

    /// Get the current class name if inside a class.
    #[inline]
    pub fn current_class_name(&self) -> Option<Atom> {
        self.block_context.scope.get_class_like_name()
    }

    /// Set the type of an expression.
    #[inline]
    pub fn set_expression_type<T: HasSpan>(&mut self, expr: &T, ty: TUnion) {
        self.artifacts.set_expression_type(expr, ty);
    }

    /// Set the type of a variable.
    #[inline]
    pub fn set_variable_type(&mut self, name: &str, ty: TUnion) {
        self.block_context.locals.insert(atom(name), Rc::new(ty));
    }

    /// Get mutable access to the analysis artifacts.
    #[inline]
    pub fn artifacts_mut(&mut self) -> &mut AnalysisArtifacts {
        self.artifacts
    }

    /// Get immutable access to the analysis artifacts.
    #[inline]
    pub fn artifacts(&self) -> &AnalysisArtifacts {
        self.artifacts
    }

    /// Get mutable access to the block context.
    #[inline]
    pub fn block_context_mut(&mut self) -> &mut BlockContext<'ctx> {
        self.block_context
    }

    /// Get immutable access to the block context.
    #[inline]
    pub fn block_context(&self) -> &BlockContext<'ctx> {
        self.block_context
    }
}

pub struct InvocationInfo<'ctx, 'ast, 'arena> {
    pub(crate) invocation: &'ctx Invocation<'ctx, 'ast, 'arena>,
}

impl<'ctx, 'ast, 'arena> InvocationInfo<'ctx, 'ast, 'arena> {
    pub(crate) fn new(invocation: &'ctx Invocation<'ctx, 'ast, 'arena>) -> Self {
        Self { invocation }
    }

    #[inline]
    #[must_use]
    pub fn get_argument(&self, index: usize, names: &[&str]) -> Option<&'ast Expression<'arena>> {
        get_argument(self.invocation.arguments_source, index, names)
    }

    #[inline]
    #[must_use]
    pub fn arguments(&self) -> Vec<InvocationArgument<'ast, 'arena>> {
        self.invocation.arguments_source.get_arguments()
    }

    #[inline]
    #[must_use]
    pub fn argument_count(&self) -> usize {
        self.invocation.arguments_source.get_arguments().len()
    }

    #[inline]
    #[must_use]
    pub fn has_no_arguments(&self) -> bool {
        self.invocation.arguments_source.get_arguments().is_empty()
    }

    #[inline]
    #[must_use]
    pub fn span(&self) -> Span {
        self.invocation.span
    }

    #[inline]
    #[must_use]
    pub fn inner(&self) -> &'ctx Invocation<'ctx, 'ast, 'arena> {
        self.invocation
    }

    #[inline]
    #[must_use]
    pub fn function_name(&self) -> String {
        self.invocation.target.guess_name()
    }
}

impl HasSpan for InvocationInfo<'_, '_, '_> {
    fn span(&self) -> Span {
        self.invocation.span
    }
}

fn get_argument<'ast, 'arena>(
    call_arguments: InvocationArgumentsSource<'ast, 'arena>,
    index: usize,
    names: &[&str],
) -> Option<&'ast Expression<'arena>> {
    match call_arguments {
        InvocationArgumentsSource::ArgumentList(argument_list) => {
            if let Some(Argument::Positional(argument)) = argument_list.arguments.get(index) {
                return Some(&argument.value);
            }

            for argument in &argument_list.arguments {
                if let Argument::Named(named_argument) = argument
                    && names.contains(&named_argument.name.value)
                {
                    return Some(&named_argument.value);
                }
            }

            None
        }
        InvocationArgumentsSource::PartialArgumentList(partial_argument_list) => {
            if let Some(PartialArgument::Positional(argument)) = partial_argument_list.arguments.get(index) {
                return Some(&argument.value);
            }

            for argument in &partial_argument_list.arguments {
                if let PartialArgument::Named(named_argument) = argument
                    && names.contains(&named_argument.name.value)
                {
                    return Some(&named_argument.value);
                }
            }

            None
        }
        InvocationArgumentsSource::PipeInput(pipe) => {
            if index == 0 {
                Some(pipe.input)
            } else {
                None
            }
        }
        InvocationArgumentsSource::None(_) => None,
    }
}
