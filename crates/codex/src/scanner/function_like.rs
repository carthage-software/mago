use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use mago_allocator::Arena;
use mago_names::ResolvedNames;
use mago_names::scope::NamespaceScope;
use mago_php_version::PHPVersion;
use mago_phpdoc_syntax::cst::AssertPattern;
use mago_phpdoc_syntax::cst::AssertTagValue;
use mago_phpdoc_syntax::cst::Element;
use mago_phpdoc_syntax::cst::TagValue;
use mago_phpdoc_syntax::cst::TextSegment;
use mago_phpdoc_syntax::cst::r#type::Type;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::Argument;
use mago_syntax::cst::ArrowFunction;
use mago_syntax::cst::Assignment;
use mago_syntax::cst::Binary;
use mago_syntax::cst::BinaryOperator;
use mago_syntax::cst::Block;
use mago_syntax::cst::Break;
use mago_syntax::cst::Closure;
use mago_syntax::cst::Conditional;
use mago_syntax::cst::Construct;
use mago_syntax::cst::Continue;
use mago_syntax::cst::DeclareColonDelimitedBody;
use mago_syntax::cst::DoWhile;
use mago_syntax::cst::ExitConstruct;
use mago_syntax::cst::Expression;
use mago_syntax::cst::For;
use mago_syntax::cst::ForBody;
use mago_syntax::cst::ForColonDelimitedBody;
use mago_syntax::cst::Foreach;
use mago_syntax::cst::ForeachBody;
use mago_syntax::cst::ForeachColonDelimitedBody;
use mago_syntax::cst::Function;
use mago_syntax::cst::FunctionCall;
use mago_syntax::cst::Goto;
use mago_syntax::cst::Hint;
use mago_syntax::cst::If;
use mago_syntax::cst::IfBody;
use mago_syntax::cst::IfColonDelimitedBody;
use mago_syntax::cst::Instantiation;
use mago_syntax::cst::Label;
use mago_syntax::cst::Literal;
use mago_syntax::cst::Match;
use mago_syntax::cst::Method;
use mago_syntax::cst::MethodBody;
use mago_syntax::cst::ModifierSequenceExt;
use mago_syntax::cst::Return;
use mago_syntax::cst::Statement;
use mago_syntax::cst::Switch;
use mago_syntax::cst::SwitchBody;
use mago_syntax::cst::SwitchCase;
use mago_syntax::cst::SwitchDefaultCase;
use mago_syntax::cst::SwitchExpressionCase;
use mago_syntax::cst::Throw;
use mago_syntax::cst::Try;
use mago_syntax::cst::TryCatchClause;
use mago_syntax::cst::UnaryPrefixOperator;
use mago_syntax::cst::Variable;
use mago_syntax::cst::While;
use mago_syntax::cst::WhileBody;
use mago_syntax::cst::WhileColonDelimitedBody;
use mago_syntax::utils;
use mago_syntax::utils::known_integer_switch_start;
use mago_syntax::utils::unparenthesized_expression;
use mago_syntax::walker::Walker;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::WordSet;
use mago_word::ascii_lowercase_word;
use mago_word::word;

use crate::assertion::Assertion;
use crate::issue::ScanningIssueKind;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::constant::ConstantMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::function_like::FunctionLikeKind;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::metadata::function_like::MethodMetadata;
use crate::metadata::parameter::CallableExceptionCatchArm;
use crate::metadata::parameter::CallableExceptionCatchGroup;
use crate::metadata::parameter::CallableExceptionCatchSummary;
use crate::metadata::parameter::CallableExceptionRethrow;
use crate::metadata::parameter::CallableInvocationKind;
use crate::misc::GenericParent;
use crate::scanner::Context;
use crate::scanner::assertion_inference::infer_assertions_from_block_body;
use crate::scanner::assertion_inference::infer_assertions_from_expression_body;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::docblock::apply_function_like_metadata_flag;
use crate::scanner::docblock::assertion_subject_word;
use crate::scanner::docblock::find_most_trusted_tag;
use crate::scanner::docblock::for_each_tag_by_ascending_trust;
use crate::scanner::docblock::parse_docblock;
use crate::scanner::parameter::scan_function_like_parameter;
use crate::scanner::parameter::scan_function_like_parameter_with_constants;
use crate::scanner::ttype::get_type_metadata_from_hint;
use crate::scanner::ttype::get_type_metadata_from_type;
use crate::scanner::ttype::merge_type_preserving_nullability;
use crate::scanner::typing_error_issue;
use crate::scanner::version_claim::evaluate_version_attributes;
use crate::ttype::builder;
use crate::ttype::builder::reject_unresolvable_tag_variable;
use crate::ttype::get_mixed;
use crate::ttype::resolution::TypeResolutionContext;
use crate::ttype::template::GenericTemplate;
use crate::visibility::Visibility;

#[inline]
pub fn scan_method<'arena, A>(
    functionlike_id: (Word, Word),
    method: &'arena Method<'arena>,
    class_like_metadata: &ClassLikeMetadata,
    context: &mut Context<'_, 'arena, A>,
    scope: &NamespaceScope,
    type_resolution_context: Option<TypeResolutionContext>,
) -> Option<FunctionLikeMetadata>
where
    A: Arena,
{
    let span = method.span();

    let mut flags = MetadataFlags::origin_flags(context.file.file_type);

    if method.ampersand.is_some() {
        flags |= MetadataFlags::BY_REFERENCE;
    }

    let verdict = evaluate_version_attributes(&method.attribute_lists, context, context.php_version);

    let lookup_name = ascii_lowercase_word(method.name.value);
    let display_name = word(method.name.value);

    let mut metadata = FunctionLikeMetadata::new(FunctionLikeKind::Method, lookup_name, display_name, span, flags);
    metadata.version_constraint = verdict.constraint;
    metadata.attributes =
        scan_attribute_lists(&method.attribute_lists, context, scope, Some(class_like_metadata.original_name));
    metadata.type_resolution_context = type_resolution_context.filter(|c| !c.is_empty());

    metadata.name_span = Some(method.name.span);
    metadata.parameters = method
        .parameter_list
        .parameters
        .iter()
        .filter_map(|p| scan_function_like_parameter(p, Some(class_like_metadata.original_name), context, scope))
        .collect();

    if let Some(return_hint) = method.return_type_hint.as_ref() {
        metadata.set_return_type_declaration_metadata(Some(get_type_metadata_from_hint(
            &return_hint.hint,
            Some(class_like_metadata.original_name),
            context,
        )));
    }

    let method_name_str = method.name.value;

    let mut method_metadata = MethodMetadata {
        is_final: method.modifiers.contains_final(),
        is_abstract: method.modifiers.contains_abstract(),
        is_static: method.modifiers.contains_static(),
        is_constructor: method_name_str.eq_ignore_ascii_case(b"__construct"),
        visibility: if let Some(v) = method.modifiers.get_first_visibility() {
            Visibility::try_from(v).unwrap_or(Visibility::Public)
        } else {
            Visibility::Public
        },
        where_constraints: WordMap::default(),
    };

    if let MethodBody::Concrete(block) = &method.body {
        if utils::block_has_yield(block) {
            metadata.flags |= MetadataFlags::HAS_YIELD;
        }

        if utils::block_has_throws(block) {
            metadata.flags |= MetadataFlags::HAS_THROW;
        }

        collect_globals_into(block, &mut metadata.globals_accessed);
    } else {
        method_metadata.is_abstract = true;
    }

    metadata.method_metadata = Some(method_metadata);

    scan_function_like_docblock(
        span,
        functionlike_id,
        &mut metadata,
        Some(class_like_metadata.original_name),
        context,
        scope,
    );

    if let MethodBody::Concrete(block) = &method.body {
        scan_callable_exception_catches(
            &mut metadata,
            context.resolved_names,
            Some(class_like_metadata.original_name),
            context.php_version,
            |walker, scan| walker.walk_block(block, scan),
        );
    }

    if let MethodBody::Concrete(block) = &method.body {
        infer_assertions_from_block_body(block, &mut metadata, context.resolved_names);
    }

    if metadata.attributes.iter().any(|attr| attr.name.as_bytes().eq_ignore_ascii_case(b"Deprecated")) {
        metadata.flags |= MetadataFlags::DEPRECATED;
    }

    // Automatically mark known fiber-suspending methods.
    if method.name.value.eq_ignore_ascii_case(b"suspend")
        && class_like_metadata.name.as_bytes().eq_ignore_ascii_case(b"revolt\\eventloop\\suspension")
    {
        metadata.flags |= MetadataFlags::SUSPENDS_FIBER;
    }

    if class_like_metadata.flags.forbids_named_arguments() {
        metadata.flags |= MetadataFlags::NO_NAMED_ARGUMENTS;
    }

    Some(metadata)
}

#[inline]
pub fn scan_function<'arena, A>(
    functionlike_id: (Word, Word),
    function: &'arena Function<'arena>,
    classname: Option<Word>,
    context: &mut Context<'_, 'arena, A>,
    scope: &NamespaceScope,
    type_resolution_context: TypeResolutionContext,
    constants: Option<&WordMap<ConstantMetadata>>,
) -> Option<FunctionLikeMetadata>
where
    A: Arena,
{
    let verdict = evaluate_version_attributes(&function.attribute_lists, context, context.php_version);

    let mut flags = MetadataFlags::origin_flags(context.file.file_type);

    if utils::block_has_yield(&function.body) {
        flags |= MetadataFlags::HAS_YIELD;
    }

    if utils::block_has_throws(&function.body) {
        flags |= MetadataFlags::HAS_THROW;
    }

    if function.ampersand.is_some() {
        flags |= MetadataFlags::BY_REFERENCE;
    }

    let name = context.resolved_names.get(&function.name);
    let lookup_name = ascii_lowercase_word(name);
    let display_name = word(name);

    let mut metadata =
        FunctionLikeMetadata::new(FunctionLikeKind::Function, lookup_name, display_name, function.span(), flags);
    metadata.version_constraint = verdict.constraint;
    collect_globals_into(&function.body, &mut metadata.globals_accessed);

    metadata.name_span = Some(function.name.span);
    metadata.parameters = function
        .parameter_list
        .parameters
        .iter()
        .filter_map(|p| scan_function_like_parameter_with_constants(p, classname, context, scope, constants))
        .collect();

    metadata.attributes = scan_attribute_lists(&function.attribute_lists, context, scope, classname);
    metadata.type_resolution_context =
        if type_resolution_context.is_empty() { None } else { Some(type_resolution_context) };

    if let Some(return_hint) = function.return_type_hint.as_ref() {
        metadata.set_return_type_declaration_metadata(Some(get_type_metadata_from_hint(
            &return_hint.hint,
            classname,
            context,
        )));
    }

    scan_function_like_docblock(function.span(), functionlike_id, &mut metadata, classname, context, scope);

    scan_callable_exception_catches(
        &mut metadata,
        context.resolved_names,
        classname,
        context.php_version,
        |walker, scan| walker.walk_block(&function.body, scan),
    );

    infer_assertions_from_block_body(&function.body, &mut metadata, context.resolved_names);

    if metadata.attributes.iter().any(|attr| attr.name.as_bytes().eq_ignore_ascii_case(b"Deprecated")) {
        metadata.flags |= MetadataFlags::DEPRECATED;
    }

    Some(metadata)
}

#[inline]
pub fn scan_closure<'arena, A>(
    functionlike_id: (Word, Word),
    closure: &'arena Closure<'arena>,
    classname: Option<Word>,
    context: &mut Context<'_, 'arena, A>,
    scope: &NamespaceScope,
    type_resolution_context: TypeResolutionContext,
) -> FunctionLikeMetadata
where
    A: Arena,
{
    let span = closure.span();

    let mut flags = MetadataFlags::origin_flags(context.file.file_type);

    if utils::block_has_yield(&closure.body) {
        flags |= MetadataFlags::HAS_YIELD;
    }

    if utils::block_has_throws(&closure.body) {
        flags |= MetadataFlags::HAS_THROW;
    }

    if closure.ampersand.is_some() {
        flags |= MetadataFlags::BY_REFERENCE;
    }

    let synthetic_name = functionlike_id.1;

    let mut metadata =
        FunctionLikeMetadata::new(FunctionLikeKind::Closure, synthetic_name, synthetic_name, span, flags)
            .with_parameters(
                closure
                    .parameter_list
                    .parameters
                    .iter()
                    .filter_map(|p| scan_function_like_parameter(p, classname, context, scope)),
            );
    collect_globals_into(&closure.body, &mut metadata.globals_accessed);

    metadata.attributes = scan_attribute_lists(&closure.attribute_lists, context, scope, classname);
    metadata.type_resolution_context =
        if type_resolution_context.is_empty() { None } else { Some(type_resolution_context) };

    if let Some(return_hint) = closure.return_type_hint.as_ref() {
        metadata.set_return_type_declaration_metadata(Some(get_type_metadata_from_hint(
            &return_hint.hint,
            classname,
            context,
        )));
    }

    scan_function_like_docblock(span, functionlike_id, &mut metadata, classname, context, scope);

    scan_callable_exception_catches(
        &mut metadata,
        context.resolved_names,
        classname,
        context.php_version,
        |walker, scan| walker.walk_block(&closure.body, scan),
    );

    infer_assertions_from_block_body(&closure.body, &mut metadata, context.resolved_names);

    metadata
}

#[inline]
pub fn scan_arrow_function<'arena, A>(
    functionlike_id: (Word, Word),
    arrow_function: &'arena ArrowFunction<'arena>,
    classname: Option<Word>,
    context: &mut Context<'_, 'arena, A>,
    scope: &NamespaceScope,
    type_resolution_context: TypeResolutionContext,
) -> FunctionLikeMetadata
where
    A: Arena,
{
    let span = arrow_function.span();

    let mut flags = MetadataFlags::origin_flags(context.file.file_type);

    if utils::expression_has_yield(arrow_function.expression) {
        flags |= MetadataFlags::HAS_YIELD;
    }

    if utils::expression_has_throws(arrow_function.expression) {
        flags |= MetadataFlags::HAS_THROW;
    }

    if arrow_function.ampersand.is_some() {
        flags |= MetadataFlags::BY_REFERENCE;
    }

    let synthetic_name = functionlike_id.1;

    let mut metadata =
        FunctionLikeMetadata::new(FunctionLikeKind::ArrowFunction, synthetic_name, synthetic_name, span, flags)
            .with_parameters(
                arrow_function
                    .parameter_list
                    .parameters
                    .iter()
                    .filter_map(|p| scan_function_like_parameter(p, classname, context, scope)),
            );

    metadata.attributes = scan_attribute_lists(&arrow_function.attribute_lists, context, scope, classname);
    metadata.type_resolution_context =
        if type_resolution_context.is_empty() { None } else { Some(type_resolution_context) };

    if let Some(return_hint) = arrow_function.return_type_hint.as_ref() {
        metadata.set_return_type_declaration_metadata(Some(get_type_metadata_from_hint(
            &return_hint.hint,
            classname,
            context,
        )));
    }

    scan_function_like_docblock(span, functionlike_id, &mut metadata, classname, context, scope);

    scan_callable_exception_catches(
        &mut metadata,
        context.resolved_names,
        classname,
        context.php_version,
        |walker, scan| walker.walk_expression(arrow_function.expression, scan),
    );

    infer_assertions_from_expression_body(arrow_function.expression, &mut metadata, context.resolved_names);

    metadata
}

struct CallbackInvocationScan<'arena> {
    parameter_name: Word,
    resolved_names: &'arena ResolvedNames<'arena>,
    classname: Option<Word>,
    php_version: PHPVersion,
    synchronously_invoked_callbacks: Vec<Span>,
    inherited_global_fallback_functions: Vec<Word>,
    catch_groups: Vec<CallableExceptionCatchGroup>,
    invocation_catches: Vec<CallableExceptionCatchSummary>,
    invocation_catch_global_fallbacks: Vec<(usize, Word)>,
    has_unresolved_catch_type: bool,
}

struct CallbackInvocationWalker;

struct CatchEscapeWalker;
struct SwitchControlTransferWalker;
struct GotoStatementWalker;

#[derive(Default)]
struct SwitchControlTransferScan {
    nested_breakable_depth: usize,
    has_transfer_to_after_switch: bool,
}

struct GotoStatementScan<'arena> {
    resolved_names: &'arena ResolvedNames<'arena>,
    classname: Option<Word>,
    termination_cache: RefCell<HashMap<Span, bool>>,
    statement_spans: Vec<Span>,
    statements_containing_goto: HashSet<Span>,
    goto_targets: HashSet<Word>,
    labels: WordMap<Span>,
}

struct CatchEscapeScan<'arena> {
    resolved_names: &'arena ResolvedNames<'arena>,
    classname: Option<Word>,
    goto_statements: Rc<HashSet<Span>>,
    goto_targets: Rc<HashSet<Word>>,
    goto_labels: Rc<WordMap<Span>>,
    termination_cache: Rc<RefCell<HashMap<Span, bool>>>,
    catch_variables: WordMap<CatchVariableException>,
    conditional_depth: usize,
    rethrows: bool,
    rethrows_types: Vec<Word>,
    rethrows_caught_types: Vec<Word>,
    rethrows_with_catches: Vec<CallableExceptionRethrow>,
}

#[derive(Clone, PartialEq, Eq)]
enum CatchVariableException {
    Caught(Vec<Word>),
    Known(Word),
    Unknown,
}

impl<'ast, 'arena> Walker<'ast, 'arena, CatchEscapeScan<'arena>> for CatchEscapeWalker {
    fn walk_block(&self, block: &'ast Block<'arena>, scan: &mut CatchEscapeScan<'arena>) {
        walk_catch_escape_statement_list(self, block.statements.as_slice(), scan);
    }

    fn walk_declare_colon_delimited_body(
        &self,
        body: &'ast DeclareColonDelimitedBody<'arena>,
        scan: &mut CatchEscapeScan<'arena>,
    ) {
        walk_catch_escape_statement_list(self, body.statements.as_slice(), scan);
    }

    fn walk_foreach_colon_delimited_body(
        &self,
        body: &'ast ForeachColonDelimitedBody<'arena>,
        scan: &mut CatchEscapeScan<'arena>,
    ) {
        walk_catch_escape_statement_list(self, body.statements.as_slice(), scan);
    }

    fn walk_for_colon_delimited_body(
        &self,
        body: &'ast ForColonDelimitedBody<'arena>,
        scan: &mut CatchEscapeScan<'arena>,
    ) {
        walk_catch_escape_statement_list(self, body.statements.as_slice(), scan);
    }

    fn walk_while_colon_delimited_body(
        &self,
        body: &'ast WhileColonDelimitedBody<'arena>,
        scan: &mut CatchEscapeScan<'arena>,
    ) {
        walk_catch_escape_statement_list(self, body.statements.as_slice(), scan);
    }

    fn walk_if_colon_delimited_body(
        &self,
        body: &'ast IfColonDelimitedBody<'arena>,
        scan: &mut CatchEscapeScan<'arena>,
    ) {
        walk_catch_escape_statement_list(self, body.statements.as_slice(), scan);

        for clause in &body.else_if_clauses {
            scan.conditional_depth += 1;
            self.walk_expression(clause.condition, scan);
            walk_catch_escape_statement_list(self, clause.statements.as_slice(), scan);
            scan.conditional_depth -= 1;
        }

        if let Some(clause) = &body.else_clause {
            walk_catch_escape_statement_list(self, clause.statements.as_slice(), scan);
        }
    }

    fn walk_throw(&self, throw: &'ast Throw<'arena>, scan: &mut CatchEscapeScan<'arena>) {
        match unparenthesized_expression(throw.exception) {
            Expression::Instantiation(Instantiation { class, .. }) => {
                let Expression::Identifier(class) = unparenthesized_expression(class) else {
                    scan.rethrows = true;
                    return;
                };
                if let Some(class_name) = scan.resolved_names.resolve(class) {
                    let exception = word(class_name);
                    if !scan.rethrows_types.contains(&exception) {
                        scan.rethrows_types.push(exception);
                    }
                } else {
                    scan.rethrows = true;
                }
            }
            Expression::Variable(Variable::Direct(variable)) => {
                let variable_name = word(normalize_variable_name(variable.name));
                match scan.catch_variables.get(&variable_name) {
                    Some(CatchVariableException::Caught(exception_types)) => {
                        for exception in exception_types {
                            if !scan.rethrows_caught_types.contains(exception) {
                                scan.rethrows_caught_types.push(*exception);
                            }
                        }
                    }
                    Some(CatchVariableException::Known(exception)) => {
                        if !scan.rethrows_types.contains(exception) {
                            scan.rethrows_types.push(*exception);
                        }
                    }
                    Some(CatchVariableException::Unknown) | None => scan.rethrows = true,
                }
            }
            _ => scan.rethrows = true,
        }
    }

    fn walk_return(&self, r#return: &'ast Return<'arena>, scan: &mut CatchEscapeScan<'arena>) {
        mago_syntax::walker::walk_return(self, r#return, scan);
    }

    fn walk_exit_construct(&self, exit_construct: &'ast ExitConstruct<'arena>, scan: &mut CatchEscapeScan<'arena>) {
        mago_syntax::walker::walk_exit_construct(self, exit_construct, scan);
    }

    fn walk_assignment(&self, assignment: &'ast Assignment<'arena>, scan: &mut CatchEscapeScan<'arena>) {
        mago_syntax::walker::walk_assignment(self, assignment, scan);

        if let Expression::Variable(Variable::Direct(variable)) = assignment.lhs {
            let variable_name = word(normalize_variable_name(variable.name));
            let value = if scan.conditional_depth > 0 || !assignment.operator.is_assign() {
                CatchVariableException::Unknown
            } else {
                match unparenthesized_expression(assignment.rhs) {
                    Expression::Instantiation(Instantiation { class, .. }) => {
                        if let Expression::Identifier(class) = unparenthesized_expression(class) {
                            scan.resolved_names
                                .resolve(class)
                                .map(|class_name| CatchVariableException::Known(word(class_name)))
                                .unwrap_or(CatchVariableException::Unknown)
                        } else {
                            CatchVariableException::Unknown
                        }
                    }
                    Expression::Variable(Variable::Direct(source)) => scan
                        .catch_variables
                        .get(&word(normalize_variable_name(source.name)))
                        .cloned()
                        .unwrap_or(CatchVariableException::Unknown),
                    _ => CatchVariableException::Unknown,
                }
            };
            scan.catch_variables.insert(variable_name, value);
        }
    }

    fn walk_if(&self, if_statement: &'ast If<'arena>, scan: &mut CatchEscapeScan<'arena>) {
        scan.conditional_depth += 1;
        mago_syntax::walker::walk_if(self, if_statement, scan);
        scan.conditional_depth -= 1;
    }

    fn walk_while(&self, r#while: &'ast While<'arena>, scan: &mut CatchEscapeScan<'arena>) {
        scan.conditional_depth += 1;
        mago_syntax::walker::walk_while(self, r#while, scan);
        scan.conditional_depth -= 1;
    }

    fn walk_do_while(&self, do_while: &'ast DoWhile<'arena>, scan: &mut CatchEscapeScan<'arena>) {
        scan.conditional_depth += 1;
        mago_syntax::walker::walk_do_while(self, do_while, scan);
        scan.conditional_depth -= 1;
    }

    fn walk_for(&self, r#for: &'ast For<'arena>, scan: &mut CatchEscapeScan<'arena>) {
        scan.conditional_depth += 1;
        mago_syntax::walker::walk_for(self, r#for, scan);
        scan.conditional_depth -= 1;
    }

    fn walk_foreach(&self, foreach: &'ast Foreach<'arena>, scan: &mut CatchEscapeScan<'arena>) {
        scan.conditional_depth += 1;
        mago_syntax::walker::walk_foreach(self, foreach, scan);
        scan.conditional_depth -= 1;
    }

    fn walk_switch(&self, switch: &'ast Switch<'arena>, scan: &mut CatchEscapeScan<'arena>) {
        scan.conditional_depth += 1;
        self.walk_expression(switch.expression, scan);
        let cases = switch.body.cases();
        let start_index = known_integer_switch_start(switch);
        let (start_index, is_known_switch) = match start_index {
            Some(Some(start_index)) => (start_index, true),
            Some(None) => {
                scan.conditional_depth -= 1;
                return;
            }
            None => (0, false),
        };
        for (index, case) in cases.iter().enumerate().skip(start_index) {
            if (!is_known_switch || index == start_index)
                && let Some(expression) = case.expression()
            {
                self.walk_expression(expression, scan);
            }
            walk_catch_escape_statement_list(self, case.statements(), scan);

            if is_known_switch
                && switch_case_stops_fallthrough(
                    switch.span(),
                    case.statements(),
                    scan.resolved_names,
                    scan.classname,
                    &scan.goto_statements,
                    &scan.goto_targets,
                    &scan.goto_labels,
                    &scan.termination_cache,
                )
            {
                break;
            }
        }
        scan.conditional_depth -= 1;
    }

    fn walk_conditional(&self, conditional: &'ast Conditional<'arena>, scan: &mut CatchEscapeScan<'arena>) {
        scan.conditional_depth += 1;
        mago_syntax::walker::walk_conditional(self, conditional, scan);
        scan.conditional_depth -= 1;
    }

    fn walk_match(&self, r#match: &'ast Match<'arena>, scan: &mut CatchEscapeScan<'arena>) {
        scan.conditional_depth += 1;
        mago_syntax::walker::walk_match(self, r#match, scan);
        scan.conditional_depth -= 1;
    }

    fn walk_binary(&self, binary: &'ast Binary<'arena>, scan: &mut CatchEscapeScan<'arena>) {
        let rhs_may_not_be_evaluated = matches!(
            binary.operator,
            BinaryOperator::And(_)
                | BinaryOperator::Or(_)
                | BinaryOperator::LowAnd(_)
                | BinaryOperator::LowOr(_)
                | BinaryOperator::NullCoalesce(_)
        );

        self.walk_expression(binary.lhs, scan);
        self.walk_binary_operator(&binary.operator, scan);

        if rhs_may_not_be_evaluated {
            scan.conditional_depth += 1;
        }

        self.walk_expression(binary.rhs, scan);

        if rhs_may_not_be_evaluated {
            scan.conditional_depth -= 1;
        }
    }

    fn walk_function(&self, _: &'ast Function<'arena>, _: &mut CatchEscapeScan<'arena>) {}

    fn walk_closure(&self, _: &'ast Closure<'arena>, _: &mut CatchEscapeScan<'arena>) {}

    fn walk_arrow_function(&self, _: &'ast ArrowFunction<'arena>, _: &mut CatchEscapeScan<'arena>) {}

    fn walk_method(&self, _: &'ast Method<'arena>, _: &mut CatchEscapeScan<'arena>) {}

    fn walk_try(&self, r#try: &'ast Try<'arena>, scan: &mut CatchEscapeScan<'arena>) {
        let reachable_catches =
            reachable_catch_clauses(&r#try.block, r#try.catch_clauses.as_slice(), scan.resolved_names, scan.classname);
        let has_catches = !reachable_catches.is_empty();
        let mut try_scan = CatchEscapeScan {
            resolved_names: scan.resolved_names,
            classname: scan.classname,
            goto_statements: Rc::clone(&scan.goto_statements),
            goto_targets: Rc::clone(&scan.goto_targets),
            goto_labels: Rc::clone(&scan.goto_labels),
            termination_cache: Rc::clone(&scan.termination_cache),
            catch_variables: scan.catch_variables.clone(),
            conditional_depth: scan.conditional_depth + usize::from(has_catches),
            rethrows: false,
            rethrows_types: Vec::new(),
            rethrows_caught_types: Vec::new(),
            rethrows_with_catches: Vec::new(),
        };
        self.walk_block(&r#try.block, &mut try_scan);

        if has_catches {
            for (variable, exception) in &try_scan.catch_variables {
                if scan.catch_variables.get(variable) != Some(exception) {
                    scan.catch_variables.insert(*variable, CatchVariableException::Unknown);
                }
            }
        } else {
            scan.catch_variables.clone_from(&try_scan.catch_variables);
        }

        let has_unresolved_catch_type = reachable_catches
            .iter()
            .any(|catch| collect_caught_class_names(&catch.hint, scan.resolved_names, scan.classname).is_none());
        let caught_types = if has_unresolved_catch_type {
            Vec::new()
        } else {
            reachable_catches
                .iter()
                .filter_map(|catch| collect_caught_class_names(&catch.hint, scan.resolved_names, scan.classname))
                .flatten()
                .collect::<Vec<_>>()
        };

        scan.rethrows |= try_scan.rethrows;
        for (exception, caught_variable) in try_scan
            .rethrows_types
            .into_iter()
            .map(|exception| (exception, false))
            .chain(try_scan.rethrows_caught_types.into_iter().map(|exception| (exception, true)))
        {
            if caught_types.is_empty() {
                let target = if caught_variable { &mut scan.rethrows_caught_types } else { &mut scan.rethrows_types };
                if !target.contains(&exception) {
                    target.push(exception);
                }
            } else {
                let rethrow = CallableExceptionRethrow {
                    exception_type: exception,
                    caught_types: caught_types.clone(),
                    caught_variable,
                };
                if !scan.rethrows_with_catches.contains(&rethrow) {
                    scan.rethrows_with_catches.push(rethrow);
                }
            }
        }

        for mut rethrow in try_scan.rethrows_with_catches {
            for caught_type in &caught_types {
                if !rethrow.caught_types.contains(caught_type) {
                    rethrow.caught_types.push(*caught_type);
                }
            }
            if !scan.rethrows_with_catches.contains(&rethrow) {
                scan.rethrows_with_catches.push(rethrow);
            }
        }

        for catch in reachable_catches {
            scan.conditional_depth += usize::from(has_catches);
            if let Some(variable) = &catch.variable {
                let variable_name = word(normalize_variable_name(variable.name));
                let exception = collect_caught_class_names(&catch.hint, scan.resolved_names, scan.classname)
                    .map(CatchVariableException::Caught)
                    .unwrap_or(CatchVariableException::Unknown);
                let previous = scan.catch_variables.insert(variable_name, exception);
                self.walk_block(&catch.block, scan);
                if let Some(previous) = previous {
                    scan.catch_variables.insert(variable_name, previous);
                } else {
                    scan.catch_variables.remove(&variable_name);
                }
            } else {
                self.walk_block(&catch.block, scan);
            }
            scan.conditional_depth -= usize::from(has_catches);
        }

        if let Some(finally) = &r#try.finally_clause {
            self.walk_block(&finally.block, scan);
        }
    }
}

fn catch_escape_summary<'arena>(
    catch: &TryCatchClause<'arena>,
    finally: Option<&Block<'arena>>,
    resolved_names: &'arena ResolvedNames<'arena>,
    classname: Option<Word>,
) -> (bool, Vec<Word>, Vec<Word>, Vec<CallableExceptionRethrow>) {
    let mut goto_scan = GotoStatementScan {
        resolved_names,
        classname,
        termination_cache: RefCell::new(HashMap::new()),
        statement_spans: Vec::new(),
        statements_containing_goto: HashSet::new(),
        goto_targets: HashSet::new(),
        labels: WordMap::default(),
    };
    let mut previous_goto_targets = HashSet::new();
    let mut previous_goto_statements = HashSet::new();
    loop {
        goto_scan.statement_spans.clear();
        goto_scan.statements_containing_goto.clear();
        goto_scan.termination_cache.get_mut().clear();

        GotoStatementWalker.walk_block(&catch.block, &mut goto_scan);
        if let Some(finally) = finally {
            GotoStatementWalker.walk_block(finally, &mut goto_scan);
        }

        if goto_scan.goto_targets == previous_goto_targets
            && goto_scan.statements_containing_goto == previous_goto_statements
        {
            break;
        }

        previous_goto_targets.clone_from(&goto_scan.goto_targets);
        previous_goto_statements.clone_from(&goto_scan.statements_containing_goto);
    }
    let goto_statements = Rc::new(goto_scan.statements_containing_goto);
    let goto_targets = Rc::new(goto_scan.goto_targets);
    let goto_labels = Rc::new(goto_scan.labels);
    let termination_cache = Rc::new(RefCell::new(HashMap::new()));

    let mut catch_scan = CatchEscapeScan {
        resolved_names,
        classname,
        goto_statements: Rc::clone(&goto_statements),
        goto_targets: Rc::clone(&goto_targets),
        goto_labels: Rc::clone(&goto_labels),
        termination_cache: Rc::clone(&termination_cache),
        catch_variables: WordMap::default(),
        conditional_depth: 0,
        rethrows: false,
        rethrows_types: Vec::new(),
        rethrows_caught_types: Vec::new(),
        rethrows_with_catches: Vec::new(),
    };
    if let Some(variable) = &catch.variable {
        let exception = collect_caught_class_names(&catch.hint, resolved_names, classname)
            .map(CatchVariableException::Caught)
            .unwrap_or(CatchVariableException::Unknown);
        catch_scan.catch_variables.insert(word(normalize_variable_name(variable.name)), exception);
    }
    CatchEscapeWalker.walk_block(&catch.block, &mut catch_scan);

    let Some(finally) = finally else {
        return (
            catch_scan.rethrows,
            catch_scan.rethrows_types,
            catch_scan.rethrows_caught_types,
            catch_scan.rethrows_with_catches,
        );
    };

    let mut finally_scan = CatchEscapeScan {
        resolved_names,
        classname,
        goto_statements: Rc::clone(&goto_statements),
        goto_targets: Rc::clone(&goto_targets),
        goto_labels: Rc::clone(&goto_labels),
        termination_cache: Rc::clone(&termination_cache),
        catch_variables: catch_scan.catch_variables,
        conditional_depth: 0,
        rethrows: false,
        rethrows_types: Vec::new(),
        rethrows_caught_types: Vec::new(),
        rethrows_with_catches: Vec::new(),
    };
    CatchEscapeWalker.walk_block(finally, &mut finally_scan);

    if block_unconditionally_terminates(finally, resolved_names, classname, &goto_statements, &termination_cache) {
        return (
            finally_scan.rethrows,
            finally_scan.rethrows_types,
            finally_scan.rethrows_caught_types,
            finally_scan.rethrows_with_catches,
        );
    }

    catch_scan.rethrows |= finally_scan.rethrows;
    for exception in finally_scan.rethrows_types {
        if !catch_scan.rethrows_types.contains(&exception) {
            catch_scan.rethrows_types.push(exception);
        }
    }
    for exception in finally_scan.rethrows_caught_types {
        if !catch_scan.rethrows_caught_types.contains(&exception) {
            catch_scan.rethrows_caught_types.push(exception);
        }
    }
    for rethrow in finally_scan.rethrows_with_catches {
        if !catch_scan.rethrows_with_catches.contains(&rethrow) {
            catch_scan.rethrows_with_catches.push(rethrow);
        }
    }

    (catch_scan.rethrows, catch_scan.rethrows_types, catch_scan.rethrows_caught_types, catch_scan.rethrows_with_catches)
}

fn block_unconditionally_terminates(
    block: &Block<'_>,
    resolved_names: &ResolvedNames<'_>,
    classname: Option<Word>,
    goto_statements: &HashSet<Span>,
    termination_cache: &RefCell<HashMap<Span, bool>>,
) -> bool {
    if block.statements.iter().any(|statement| matches!(statement, Statement::Goto(_))) {
        let label_indices = block
            .statements
            .iter()
            .enumerate()
            .filter_map(|(index, statement)| match statement {
                Statement::Label(label) => Some((word(label.name.value), index)),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        let mut pending_indices = vec![0];
        let mut visited_indices = HashSet::new();

        while let Some(index) = pending_indices.pop() {
            if index == block.statements.len() {
                return false;
            }

            if !visited_indices.insert(index) {
                continue;
            }

            let statement = &block.statements[index];
            if let Statement::Goto(goto) = statement {
                let Some(target) = label_indices.get(&word(goto.label.value)) else {
                    return false;
                };

                pending_indices.push(*target);

                continue;
            }

            if goto_statements.contains(&statement.span()) {
                return false;
            }

            if !statement_unconditionally_terminates(
                statement,
                resolved_names,
                classname,
                goto_statements,
                termination_cache,
            ) {
                pending_indices.push(index + 1);
            }
        }

        return true;
    }

    for statement in &block.statements {
        if goto_statements.contains(&statement.span()) {
            return false;
        }

        if statement_unconditionally_terminates(
            statement,
            resolved_names,
            classname,
            goto_statements,
            termination_cache,
        ) {
            return true;
        }
    }

    false
}

fn statement_unconditionally_terminates(
    statement: &Statement<'_>,
    resolved_names: &ResolvedNames<'_>,
    classname: Option<Word>,
    goto_statements: &HashSet<Span>,
    termination_cache: &RefCell<HashMap<Span, bool>>,
) -> bool {
    let span = statement.span();
    if let Some(terminates) = termination_cache.borrow().get(&span).copied() {
        return terminates;
    }

    let terminates = !goto_statements.contains(&span)
        && statement_unconditionally_terminates_uncached(
            statement,
            resolved_names,
            classname,
            goto_statements,
            termination_cache,
        );
    termination_cache.borrow_mut().insert(span, terminates);
    terminates
}

fn statement_unconditionally_terminates_uncached(
    statement: &Statement<'_>,
    resolved_names: &ResolvedNames<'_>,
    classname: Option<Word>,
    goto_statements: &HashSet<Span>,
    termination_cache: &RefCell<HashMap<Span, bool>>,
) -> bool {
    match statement {
        Statement::Block(block) => {
            block_unconditionally_terminates(block, resolved_names, classname, goto_statements, termination_cache)
        }
        Statement::Expression(expression) => expression_unconditionally_terminates(expression.expression),
        Statement::Return(_) => true,
        Statement::If(r#if) => {
            let body = &r#if.body;
            body.statements().iter().any(|statement| {
                statement_unconditionally_terminates(
                    statement,
                    resolved_names,
                    classname,
                    goto_statements,
                    termination_cache,
                )
            }) && body.else_if_statements().iter().all(|statements| {
                statements.iter().any(|statement| {
                    statement_unconditionally_terminates(
                        statement,
                        resolved_names,
                        classname,
                        goto_statements,
                        termination_cache,
                    )
                })
            }) && body.else_statements().is_some_and(|statements| {
                statements.iter().any(|statement| {
                    statement_unconditionally_terminates(
                        statement,
                        resolved_names,
                        classname,
                        goto_statements,
                        termination_cache,
                    )
                })
            })
        }
        Statement::Try(r#try) => {
            r#try.finally_clause.as_ref().is_some_and(|finally| {
                block_unconditionally_terminates(
                    &finally.block,
                    resolved_names,
                    classname,
                    goto_statements,
                    termination_cache,
                )
            }) || (block_unconditionally_terminates(
                &r#try.block,
                resolved_names,
                classname,
                goto_statements,
                termination_cache,
            ) && (reachable_catch_clauses(
                &r#try.block,
                r#try.catch_clauses.as_slice(),
                resolved_names,
                classname,
            )
            .iter()
            .all(|catch| {
                block_unconditionally_terminates(
                    &catch.block,
                    resolved_names,
                    classname,
                    goto_statements,
                    termination_cache,
                )
            })))
        }
        Statement::Switch(switch) => {
            if let Some(Some(start_index)) = known_integer_switch_start(switch) {
                for case in &switch.body.cases()[start_index..] {
                    if switch_case_unconditionally_terminates(
                        case.statements(),
                        resolved_names,
                        classname,
                        goto_statements,
                        termination_cache,
                    ) {
                        return true;
                    }

                    if switch_case_stops_before_fallthrough(
                        case.statements(),
                        resolved_names,
                        classname,
                        goto_statements,
                        termination_cache,
                    ) {
                        return false;
                    }
                }

                false
            } else {
                switch.body.has_default_case()
                    && switch.body.cases().iter().all(|case| {
                        switch_case_unconditionally_terminates(
                            case.statements(),
                            resolved_names,
                            classname,
                            goto_statements,
                            termination_cache,
                        )
                    })
            }
        }
        _ => false,
    }
}

fn switch_case_stops_before_fallthrough(
    statements: &[Statement<'_>],
    resolved_names: &ResolvedNames<'_>,
    classname: Option<Word>,
    goto_statements: &HashSet<Span>,
    termination_cache: &RefCell<HashMap<Span, bool>>,
) -> bool {
    statements.iter().any(|statement| {
        if statement_unconditionally_terminates(
            statement,
            resolved_names,
            classname,
            goto_statements,
            termination_cache,
        ) {
            return true;
        }

        match statement {
            Statement::Break(_) | Statement::Continue(_) | Statement::Goto(_) => true,
            Statement::Block(block) => switch_case_stops_before_fallthrough(
                block.statements.as_slice(),
                resolved_names,
                classname,
                goto_statements,
                termination_cache,
            ),
            Statement::If(if_statement) => {
                let body = &if_statement.body;
                switch_case_stops_before_fallthrough(
                    body.statements(),
                    resolved_names,
                    classname,
                    goto_statements,
                    termination_cache,
                ) && body.else_if_statements().iter().all(|statements| {
                    switch_case_stops_before_fallthrough(
                        statements,
                        resolved_names,
                        classname,
                        goto_statements,
                        termination_cache,
                    )
                }) && body.else_statements().is_some_and(|statements| {
                    switch_case_stops_before_fallthrough(
                        statements,
                        resolved_names,
                        classname,
                        goto_statements,
                        termination_cache,
                    )
                })
            }
            Statement::Try(r#try) => {
                let try_stops = switch_case_stops_before_fallthrough(
                    r#try.block.statements.as_slice(),
                    resolved_names,
                    classname,
                    goto_statements,
                    termination_cache,
                );
                let catches_stop =
                    reachable_catch_clauses(&r#try.block, r#try.catch_clauses.as_slice(), resolved_names, classname)
                        .iter()
                        .all(|catch| {
                            switch_case_stops_before_fallthrough(
                                catch.block.statements.as_slice(),
                                resolved_names,
                                classname,
                                goto_statements,
                                termination_cache,
                            )
                        });
                let finally_stops = r#try.finally_clause.as_ref().is_some_and(|finally| {
                    switch_case_stops_before_fallthrough(
                        finally.block.statements.as_slice(),
                        resolved_names,
                        classname,
                        goto_statements,
                        termination_cache,
                    )
                });

                finally_stops || (try_stops && catches_stop)
            }
            _ => false,
        }
    })
}

fn switch_case_unconditionally_terminates(
    statements: &[Statement<'_>],
    resolved_names: &ResolvedNames<'_>,
    classname: Option<Word>,
    goto_statements: &HashSet<Span>,
    termination_cache: &RefCell<HashMap<Span, bool>>,
) -> bool {
    let mut scan = SwitchControlTransferScan::default();
    for statement in statements {
        SwitchControlTransferWalker.walk_statement(statement, &mut scan);
    }
    if scan.has_transfer_to_after_switch {
        return false;
    }

    statements.iter().any(|statement| {
        statement_unconditionally_terminates(statement, resolved_names, classname, goto_statements, termination_cache)
            || statement_unconditionally_transfers_past_switch(
                statement,
                0,
                resolved_names,
                classname,
                goto_statements,
                termination_cache,
            )
    })
}

fn switch_case_stops_fallthrough(
    switch_span: Span,
    statements: &[Statement<'_>],
    resolved_names: &ResolvedNames<'_>,
    classname: Option<Word>,
    goto_statements: &HashSet<Span>,
    goto_targets: &HashSet<Word>,
    goto_labels: &WordMap<Span>,
    termination_cache: &RefCell<HashMap<Span, bool>>,
) -> bool {
    statements.iter().any(|statement| {
        statement_stops_switch_fallthrough(
            statement,
            switch_span,
            resolved_names,
            classname,
            goto_statements,
            goto_targets,
            goto_labels,
            termination_cache,
        )
    })
}

fn statement_stops_switch_fallthrough(
    statement: &Statement<'_>,
    switch_span: Span,
    resolved_names: &ResolvedNames<'_>,
    classname: Option<Word>,
    goto_statements: &HashSet<Span>,
    goto_targets: &HashSet<Word>,
    goto_labels: &WordMap<Span>,
    termination_cache: &RefCell<HashMap<Span, bool>>,
) -> bool {
    if statement_unconditionally_terminates(statement, resolved_names, classname, goto_statements, termination_cache)
        || statement_unconditionally_transfers_past_switch(
            statement,
            0,
            resolved_names,
            classname,
            goto_statements,
            termination_cache,
        )
    {
        return true;
    }

    match statement {
        Statement::Break(r#break) => control_transfer_leaves_switch(r#break.level, 0),
        Statement::Continue(r#continue) => control_transfer_leaves_switch(r#continue.level, 0),
        Statement::Goto(goto) if goto_statements.contains(&statement.span()) => {
            let target = word(goto.label.value);
            if !goto_targets.contains(&target) {
                return false;
            }

            goto_labels.get(&target).is_some_and(|label_span| {
                label_span.file_id == switch_span.file_id
                    && !((switch_span.start.offset..switch_span.end.offset).contains(&label_span.start.offset))
            })
        }
        Statement::Block(block) => switch_case_stops_fallthrough(
            switch_span,
            block.statements.as_slice(),
            resolved_names,
            classname,
            goto_statements,
            goto_targets,
            goto_labels,
            termination_cache,
        ),
        Statement::If(if_statement) => {
            let body = &if_statement.body;
            switch_case_stops_fallthrough(
                switch_span,
                body.statements(),
                resolved_names,
                classname,
                goto_statements,
                goto_targets,
                goto_labels,
                termination_cache,
            ) && body.else_if_statements().iter().all(|statements| {
                switch_case_stops_fallthrough(
                    switch_span,
                    statements,
                    resolved_names,
                    classname,
                    goto_statements,
                    goto_targets,
                    goto_labels,
                    termination_cache,
                )
            }) && body.else_statements().is_some_and(|statements| {
                switch_case_stops_fallthrough(
                    switch_span,
                    statements,
                    resolved_names,
                    classname,
                    goto_statements,
                    goto_targets,
                    goto_labels,
                    termination_cache,
                )
            })
        }
        Statement::Try(r#try) => {
            let try_stops = switch_case_stops_fallthrough(
                switch_span,
                r#try.block.statements.as_slice(),
                resolved_names,
                classname,
                goto_statements,
                goto_targets,
                goto_labels,
                termination_cache,
            );
            let catches_stop =
                reachable_catch_clauses(&r#try.block, r#try.catch_clauses.as_slice(), resolved_names, classname)
                    .iter()
                    .all(|catch| {
                        switch_case_stops_fallthrough(
                            switch_span,
                            catch.block.statements.as_slice(),
                            resolved_names,
                            classname,
                            goto_statements,
                            goto_targets,
                            goto_labels,
                            termination_cache,
                        )
                    });
            let finally_stops = r#try.finally_clause.as_ref().is_some_and(|finally| {
                switch_case_stops_fallthrough(
                    switch_span,
                    finally.block.statements.as_slice(),
                    resolved_names,
                    classname,
                    goto_statements,
                    goto_targets,
                    goto_labels,
                    termination_cache,
                )
            });

            finally_stops || (try_stops && catches_stop)
        }
        _ => false,
    }
}

fn statement_unconditionally_transfers_past_switch(
    statement: &Statement<'_>,
    nested_breakable_depth: usize,
    resolved_names: &ResolvedNames<'_>,
    classname: Option<Word>,
    goto_statements: &HashSet<Span>,
    termination_cache: &RefCell<HashMap<Span, bool>>,
) -> bool {
    if goto_statements.contains(&statement.span()) {
        return false;
    }

    match statement {
        Statement::Try(r#try) => {
            let catches_exit =
                reachable_catch_clauses(&r#try.block, r#try.catch_clauses.as_slice(), resolved_names, classname)
                    .iter()
                    .all(|catch| {
                        block_unconditionally_exits_switch_region(
                            &catch.block,
                            nested_breakable_depth,
                            resolved_names,
                            classname,
                            goto_statements,
                            termination_cache,
                        )
                    });
            let try_exits = catches_exit
                && block_unconditionally_exits_switch_region(
                    &r#try.block,
                    nested_breakable_depth,
                    resolved_names,
                    classname,
                    goto_statements,
                    termination_cache,
                );
            let finally_exits = r#try.finally_clause.as_ref().is_some_and(|finally| {
                block_unconditionally_exits_switch_region(
                    &finally.block,
                    nested_breakable_depth,
                    resolved_names,
                    classname,
                    goto_statements,
                    termination_cache,
                )
            });

            try_exits || finally_exits
        }
        Statement::Break(r#break) => control_transfer_leaves_past_switch(r#break.level, nested_breakable_depth),
        Statement::Continue(r#continue) => {
            control_transfer_leaves_past_switch(r#continue.level, nested_breakable_depth)
        }
        Statement::Block(block) => block.statements.iter().any(|statement| {
            statement_unconditionally_transfers_past_switch(
                statement,
                nested_breakable_depth,
                resolved_names,
                classname,
                goto_statements,
                termination_cache,
            )
        }),
        Statement::If(r#if) => {
            let body = &r#if.body;
            body.statements().iter().any(|statement| {
                statement_unconditionally_terminates(
                    statement,
                    resolved_names,
                    classname,
                    goto_statements,
                    termination_cache,
                ) || statement_unconditionally_transfers_past_switch(
                    statement,
                    nested_breakable_depth,
                    resolved_names,
                    classname,
                    goto_statements,
                    termination_cache,
                )
            }) && body.else_if_statements().iter().all(|statements| {
                statements.iter().any(|statement| {
                    statement_unconditionally_terminates(
                        statement,
                        resolved_names,
                        classname,
                        goto_statements,
                        termination_cache,
                    ) || statement_unconditionally_transfers_past_switch(
                        statement,
                        nested_breakable_depth,
                        resolved_names,
                        classname,
                        goto_statements,
                        termination_cache,
                    )
                })
            }) && body.else_statements().is_some_and(|statements| {
                statements.iter().any(|statement| {
                    statement_unconditionally_terminates(
                        statement,
                        resolved_names,
                        classname,
                        goto_statements,
                        termination_cache,
                    ) || statement_unconditionally_transfers_past_switch(
                        statement,
                        nested_breakable_depth,
                        resolved_names,
                        classname,
                        goto_statements,
                        termination_cache,
                    )
                })
            })
        }
        _ => false,
    }
}

fn block_unconditionally_exits_switch_region(
    block: &Block<'_>,
    nested_breakable_depth: usize,
    resolved_names: &ResolvedNames<'_>,
    classname: Option<Word>,
    goto_statements: &HashSet<Span>,
    termination_cache: &RefCell<HashMap<Span, bool>>,
) -> bool {
    for statement in &block.statements {
        if goto_statements.contains(&statement.span()) {
            return false;
        }

        if statement_unconditionally_terminates(
            statement,
            resolved_names,
            classname,
            goto_statements,
            termination_cache,
        ) || statement_unconditionally_transfers_past_switch(
            statement,
            nested_breakable_depth,
            resolved_names,
            classname,
            goto_statements,
            termination_cache,
        ) {
            return true;
        }
    }

    false
}

fn reachable_catch_clauses<'catch, 'arena>(
    block: &Block<'_>,
    catches: &'catch [TryCatchClause<'arena>],
    resolved_names: &ResolvedNames<'_>,
    classname: Option<Word>,
) -> Vec<&'catch TryCatchClause<'arena>> {
    if block_certainly_does_not_throw(block) {
        return Vec::new();
    }

    let Some(thrown_class) = directly_thrown_class(block, resolved_names) else {
        return catches.iter().collect();
    };

    let mut reachable_catches = Vec::new();
    for catch in catches {
        let Some(caught_classes) = collect_caught_class_names(&catch.hint, resolved_names, classname) else {
            reachable_catches.push(catch);
            continue;
        };
        if !caught_classes.is_empty()
            && caught_classes.iter().all(|caught_class| {
                is_builtin_exception_instance_of(thrown_class.as_bytes(), caught_class.as_bytes()) == Some(false)
            })
        {
            continue;
        }

        let definitely_catches_thrown_class = caught_classes.iter().any(|caught_class| {
            is_builtin_exception_instance_of(thrown_class.as_bytes(), caught_class.as_bytes()) == Some(true)
        });
        reachable_catches.push(catch);
        if definitely_catches_thrown_class {
            break;
        }
    }

    reachable_catches
}

fn block_certainly_does_not_throw(block: &Block<'_>) -> bool {
    for statement in &block.statements {
        match statement {
            Statement::Noop(_) | Statement::Label(_) => {}
            Statement::Block(block) if block_certainly_does_not_throw(block) => {}
            Statement::If(r#if) if if_statement_certainly_does_not_throw(r#if) => {}
            Statement::Break(r#break) if control_transfer_level_does_not_throw(r#break.level) => return true,
            Statement::Continue(r#continue) if control_transfer_level_does_not_throw(r#continue.level) => return true,
            Statement::Return(r#return) if r#return.value.is_none() => return true,
            _ => return false,
        }
    }

    true
}

fn if_statement_certainly_does_not_throw(r#if: &If<'_>) -> bool {
    match syntax_truthiness(r#if.condition) {
        Some(true) => statement_list_certainly_does_not_throw(r#if.body.statements()),
        Some(false) => match &r#if.body {
            IfBody::Statement(body) => {
                for clause in &body.else_if_clauses {
                    match syntax_truthiness(clause.condition) {
                        Some(false) => {}
                        Some(true) => {
                            return statement_list_certainly_does_not_throw(std::slice::from_ref(clause.statement));
                        }
                        None => return false,
                    }
                }

                body.else_clause.as_ref().is_none_or(|else_clause| {
                    statement_list_certainly_does_not_throw(std::slice::from_ref(else_clause.statement))
                })
            }
            IfBody::ColonDelimited(body) => {
                for clause in &body.else_if_clauses {
                    match syntax_truthiness(clause.condition) {
                        Some(false) => {}
                        Some(true) => return statement_list_certainly_does_not_throw(clause.statements.as_slice()),
                        None => return false,
                    }
                }

                body.else_clause.as_ref().is_none_or(|else_clause| {
                    statement_list_certainly_does_not_throw(else_clause.statements.as_slice())
                })
            }
        },
        None => false,
    }
}

fn statement_list_certainly_does_not_throw(statements: &[Statement<'_>]) -> bool {
    for statement in statements {
        match statement {
            Statement::Noop(_) | Statement::Label(_) => {}
            Statement::Block(block) if block_certainly_does_not_throw(block) => {}
            Statement::If(r#if) if if_statement_certainly_does_not_throw(r#if) => {}
            Statement::Break(r#break) if control_transfer_level_does_not_throw(r#break.level) => return true,
            Statement::Continue(r#continue) if control_transfer_level_does_not_throw(r#continue.level) => return true,
            Statement::Return(r#return) if r#return.value.is_none() => return true,
            _ => return false,
        }
    }

    true
}

fn control_transfer_level_does_not_throw(level: Option<&Expression<'_>>) -> bool {
    level.is_none_or(|level| matches!(unparenthesized_expression(level), Expression::Literal(Literal::Integer(_))))
}

fn directly_thrown_class(block: &Block<'_>, resolved_names: &ResolvedNames<'_>) -> Option<Word> {
    let [Statement::Expression(expression)] = block.statements.as_slice() else {
        return None;
    };
    let Expression::Throw(throw) = unparenthesized_expression(expression.expression) else {
        return None;
    };
    let Expression::Instantiation(instantiation) = unparenthesized_expression(throw.exception) else {
        return None;
    };
    let Expression::Identifier(class) = unparenthesized_expression(instantiation.class) else {
        return None;
    };

    resolved_names.resolve(class).map(word)
}

fn is_builtin_exception_instance_of(thrown_class: &[u8], caught_class: &[u8]) -> Option<bool> {
    let thrown_class = thrown_class.strip_prefix(b"\\").unwrap_or(thrown_class);
    let caught_class = caught_class.strip_prefix(b"\\").unwrap_or(caught_class);
    if thrown_class.eq_ignore_ascii_case(caught_class) {
        return Some(true);
    }

    if !is_builtin_exception_name(thrown_class) || !is_builtin_exception_name(caught_class) {
        return None;
    }

    let mut current = thrown_class;
    loop {
        if current.eq_ignore_ascii_case(caught_class) {
            return Some(true);
        }

        let Some(parent) = builtin_exception_parent(current) else {
            return Some(false);
        };
        current = parent;
    }
}

fn is_builtin_exception_name(class: &[u8]) -> bool {
    class.eq_ignore_ascii_case(b"Throwable") || builtin_exception_parent(class).is_some()
}

fn builtin_exception_parent(class: &[u8]) -> Option<&'static [u8]> {
    if class.eq_ignore_ascii_case(b"ErrorException")
        || class.eq_ignore_ascii_case(b"LogicException")
        || class.eq_ignore_ascii_case(b"RuntimeException")
    {
        return Some(b"Exception");
    }
    if class.eq_ignore_ascii_case(b"Exception") || class.eq_ignore_ascii_case(b"Error") {
        return Some(b"Throwable");
    }
    if class.eq_ignore_ascii_case(b"BadFunctionCallException") {
        return Some(b"LogicException");
    }
    if class.eq_ignore_ascii_case(b"BadMethodCallException") {
        return Some(b"BadFunctionCallException");
    }
    if class.eq_ignore_ascii_case(b"DomainException")
        || class.eq_ignore_ascii_case(b"InvalidArgumentException")
        || class.eq_ignore_ascii_case(b"LengthException")
        || class.eq_ignore_ascii_case(b"OutOfRangeException")
    {
        return Some(b"LogicException");
    }
    if class.eq_ignore_ascii_case(b"OutOfBoundsException")
        || class.eq_ignore_ascii_case(b"OverflowException")
        || class.eq_ignore_ascii_case(b"RangeException")
        || class.eq_ignore_ascii_case(b"UnderflowException")
        || class.eq_ignore_ascii_case(b"UnexpectedValueException")
    {
        return Some(b"RuntimeException");
    }

    None
}

impl<'ast, 'arena> Walker<'ast, 'arena, SwitchControlTransferScan> for SwitchControlTransferWalker {
    fn walk_break(&self, r#break: &'ast Break<'arena>, scan: &mut SwitchControlTransferScan) {
        let nested_breakable_depth = scan.nested_breakable_depth;
        let leaves_switch = control_transfer_leaves_switch(r#break.level, nested_breakable_depth);
        let leaves_only_switch =
            leaves_switch && !control_transfer_leaves_past_switch(r#break.level, nested_breakable_depth);
        scan.has_transfer_to_after_switch |= leaves_only_switch;
    }

    fn walk_continue(&self, r#continue: &'ast Continue<'arena>, scan: &mut SwitchControlTransferScan) {
        let nested_breakable_depth = scan.nested_breakable_depth;
        let leaves_switch = control_transfer_leaves_switch(r#continue.level, nested_breakable_depth);
        let leaves_only_switch =
            leaves_switch && !control_transfer_leaves_past_switch(r#continue.level, nested_breakable_depth);
        scan.has_transfer_to_after_switch |= leaves_only_switch;
    }

    fn walk_while(&self, r#while: &'ast While<'arena>, scan: &mut SwitchControlTransferScan) {
        scan.nested_breakable_depth += 1;
        mago_syntax::walker::walk_while(self, r#while, scan);
        scan.nested_breakable_depth -= 1;
    }

    fn walk_do_while(&self, do_while: &'ast DoWhile<'arena>, scan: &mut SwitchControlTransferScan) {
        scan.nested_breakable_depth += 1;
        mago_syntax::walker::walk_do_while(self, do_while, scan);
        scan.nested_breakable_depth -= 1;
    }

    fn walk_for(&self, r#for: &'ast For<'arena>, scan: &mut SwitchControlTransferScan) {
        scan.nested_breakable_depth += 1;
        mago_syntax::walker::walk_for(self, r#for, scan);
        scan.nested_breakable_depth -= 1;
    }

    fn walk_foreach(&self, foreach: &'ast Foreach<'arena>, scan: &mut SwitchControlTransferScan) {
        scan.nested_breakable_depth += 1;
        mago_syntax::walker::walk_foreach(self, foreach, scan);
        scan.nested_breakable_depth -= 1;
    }

    fn walk_switch(&self, switch: &'ast Switch<'arena>, scan: &mut SwitchControlTransferScan) {
        scan.nested_breakable_depth += 1;
        mago_syntax::walker::walk_switch(self, switch, scan);
        scan.nested_breakable_depth -= 1;
    }

    fn walk_function(&self, _: &'ast Function<'arena>, _: &mut SwitchControlTransferScan) {}

    fn walk_closure(&self, _: &'ast Closure<'arena>, _: &mut SwitchControlTransferScan) {}

    fn walk_arrow_function(&self, _: &'ast ArrowFunction<'arena>, _: &mut SwitchControlTransferScan) {}

    fn walk_method(&self, _: &'ast Method<'arena>, _: &mut SwitchControlTransferScan) {}
}

impl<'ast, 'arena> Walker<'ast, 'arena, GotoStatementScan<'arena>> for GotoStatementWalker {
    fn walk_block(&self, block: &'ast Block<'arena>, scan: &mut GotoStatementScan<'arena>) {
        walk_goto_statement_list(self, block.statements.as_slice(), scan);
    }

    fn walk_try(&self, r#try: &'ast Try<'arena>, scan: &mut GotoStatementScan<'arena>) {
        self.walk_block(&r#try.block, scan);

        for catch in
            reachable_catch_clauses(&r#try.block, r#try.catch_clauses.as_slice(), scan.resolved_names, scan.classname)
        {
            self.walk_block(&catch.block, scan);
        }

        if let Some(finally) = &r#try.finally_clause {
            self.walk_block(&finally.block, scan);
        }
    }

    fn walk_switch(&self, switch: &'ast Switch<'arena>, scan: &mut GotoStatementScan<'arena>) {
        let cases = switch.body.cases();
        let (start_index, is_known_switch) = match known_integer_switch_start(switch) {
            Some(Some(start_index)) => (start_index, true),
            Some(None) => return,
            None => (0, false),
        };

        for case in &cases[start_index..] {
            match case {
                SwitchCase::Expression(case) => self.walk_switch_expression_case(case, scan),
                SwitchCase::Default(case) => self.walk_switch_default_case(case, scan),
            }

            if is_known_switch
                && switch_case_stops_fallthrough(
                    switch.span(),
                    case.statements(),
                    scan.resolved_names,
                    scan.classname,
                    &scan.statements_containing_goto,
                    &scan.goto_targets,
                    &scan.labels,
                    &scan.termination_cache,
                )
            {
                break;
            }
        }
    }

    fn walk_declare_colon_delimited_body(
        &self,
        body: &'ast DeclareColonDelimitedBody<'arena>,
        scan: &mut GotoStatementScan<'arena>,
    ) {
        walk_goto_statement_list(self, body.statements.as_slice(), scan);
    }

    fn walk_foreach_colon_delimited_body(
        &self,
        body: &'ast ForeachColonDelimitedBody<'arena>,
        scan: &mut GotoStatementScan<'arena>,
    ) {
        walk_goto_statement_list(self, body.statements.as_slice(), scan);
    }

    fn walk_for_colon_delimited_body(
        &self,
        body: &'ast ForColonDelimitedBody<'arena>,
        scan: &mut GotoStatementScan<'arena>,
    ) {
        walk_goto_statement_list(self, body.statements.as_slice(), scan);
    }

    fn walk_while_colon_delimited_body(
        &self,
        body: &'ast WhileColonDelimitedBody<'arena>,
        scan: &mut GotoStatementScan<'arena>,
    ) {
        walk_goto_statement_list(self, body.statements.as_slice(), scan);
    }

    fn walk_switch_expression_case(
        &self,
        case: &'ast SwitchExpressionCase<'arena>,
        scan: &mut GotoStatementScan<'arena>,
    ) {
        self.walk_expression(case.expression, scan);
        walk_goto_statement_list(self, case.statements.as_slice(), scan);
    }

    fn walk_switch_default_case(&self, case: &'ast SwitchDefaultCase<'arena>, scan: &mut GotoStatementScan<'arena>) {
        walk_goto_statement_list(self, case.statements.as_slice(), scan);
    }

    fn walk_if_colon_delimited_body(
        &self,
        body: &'ast IfColonDelimitedBody<'arena>,
        scan: &mut GotoStatementScan<'arena>,
    ) {
        walk_goto_statement_list(self, body.statements.as_slice(), scan);

        for clause in &body.else_if_clauses {
            self.walk_expression(clause.condition, scan);
            walk_goto_statement_list(self, clause.statements.as_slice(), scan);
        }

        if let Some(clause) = &body.else_clause {
            walk_goto_statement_list(self, clause.statements.as_slice(), scan);
        }
    }

    fn walk_statement(&self, statement: &'ast Statement<'arena>, scan: &mut GotoStatementScan<'arena>) {
        scan.statement_spans.push(statement.span());
        mago_syntax::walker::walk_statement(self, statement, scan);
        scan.statement_spans.pop();
    }

    fn walk_goto(&self, goto: &'ast Goto<'arena>, scan: &mut GotoStatementScan<'arena>) {
        scan.statements_containing_goto.extend(scan.statement_spans.iter().copied());
        scan.goto_targets.insert(word(goto.label.value));
    }

    fn walk_label(&self, label: &'ast Label<'arena>, scan: &mut GotoStatementScan<'arena>) {
        scan.labels.insert(word(label.name.value), label.span());
    }

    fn walk_function(&self, _: &'ast Function<'arena>, _: &mut GotoStatementScan<'arena>) {}

    fn walk_closure(&self, _: &'ast Closure<'arena>, _: &mut GotoStatementScan<'arena>) {}

    fn walk_arrow_function(&self, _: &'ast ArrowFunction<'arena>, _: &mut GotoStatementScan<'arena>) {}

    fn walk_method(&self, _: &'ast Method<'arena>, _: &mut GotoStatementScan<'arena>) {}
}

fn walk_goto_statement_list<'arena>(
    walker: &GotoStatementWalker,
    statements: &[Statement<'arena>],
    scan: &mut GotoStatementScan<'arena>,
) {
    let mut falls_through = true;
    for statement in statements {
        if !falls_through {
            if !statement_is_goto_target(statement, &scan.goto_targets) {
                continue;
            }
        }

        walker.walk_statement(statement, scan);
        falls_through = !matches!(statement, Statement::Goto(_) | Statement::Break(_) | Statement::Continue(_))
            && !statement_unconditionally_terminates(
                statement,
                scan.resolved_names,
                scan.classname,
                &scan.statements_containing_goto,
                &scan.termination_cache,
            );
    }
}

fn statement_is_goto_target(statement: &Statement<'_>, goto_targets: &HashSet<Word>) -> bool {
    matches!(statement, Statement::Label(label) if goto_targets.contains(&word(label.name.value)))
}

fn walk_catch_escape_statement_list<'arena>(
    walker: &CatchEscapeWalker,
    statements: &[Statement<'arena>],
    scan: &mut CatchEscapeScan<'arena>,
) {
    let mut falls_through = true;
    for statement in statements {
        if !falls_through {
            if !statement_is_goto_target(statement, &scan.goto_targets) {
                continue;
            }
        }

        walker.walk_statement(statement, scan);
        falls_through = !matches!(statement, Statement::Goto(_) | Statement::Break(_) | Statement::Continue(_))
            && !statement_unconditionally_terminates(
                statement,
                scan.resolved_names,
                scan.classname,
                &scan.goto_statements,
                &scan.termination_cache,
            );
    }
}

fn control_transfer_leaves_switch(level: Option<&Expression<'_>>, nested_breakable_depth: usize) -> bool {
    match level {
        None => nested_breakable_depth == 0,
        Some(level) => match unparenthesized_expression(level) {
            Expression::Literal(Literal::Integer(level)) => {
                level.value.is_none_or(|level| level > nested_breakable_depth as u64)
            }
            _ => true,
        },
    }
}

fn control_transfer_leaves_past_switch(level: Option<&Expression<'_>>, nested_breakable_depth: usize) -> bool {
    match level {
        Some(level) => match unparenthesized_expression(level) {
            Expression::Literal(Literal::Integer(level)) => {
                level.value.is_some_and(|level| level > nested_breakable_depth as u64 + 1)
            }
            _ => false,
        },
        None => false,
    }
}

fn expression_unconditionally_terminates(expression: &Expression<'_>) -> bool {
    match unparenthesized_expression(expression) {
        Expression::Throw(_) | Expression::Construct(Construct::Exit(_)) => true,
        Expression::Match(r#match) => {
            r#match.arms.iter().all(|arm| expression_unconditionally_terminates(arm.expression()))
        }
        _ => false,
    }
}

fn normalize_variable_name(name: &[u8]) -> &[u8] {
    name.strip_prefix(b"$").unwrap_or(name)
}

impl<'ast, 'arena> Walker<'ast, 'arena, CallbackInvocationScan<'arena>> for CallbackInvocationWalker {
    fn walk_if(&self, r#if: &'ast If<'arena>, scan: &mut CallbackInvocationScan<'arena>) {
        self.walk_expression(r#if.condition, scan);
        self.walk_if_body_after_condition(&r#if.body, scan, syntax_truthiness(r#if.condition));
    }

    fn walk_conditional(&self, conditional: &'ast Conditional<'arena>, scan: &mut CallbackInvocationScan<'arena>) {
        self.walk_expression(conditional.condition, scan);

        match syntax_truthiness(conditional.condition) {
            Some(true) => {
                if let Some(then) = conditional.then {
                    self.walk_expression(then, scan);
                }
            }
            Some(false) => self.walk_expression(conditional.r#else, scan),
            None => {
                if let Some(then) = conditional.then {
                    self.walk_expression(then, scan);
                }
                self.walk_expression(conditional.r#else, scan);
            }
        }
    }

    fn walk_switch(&self, r#switch: &'ast Switch<'arena>, scan: &mut CallbackInvocationScan<'arena>) {
        self.walk_expression(r#switch.expression, scan);

        let Some(start_index) = known_boolean_switch_start(r#switch) else {
            self.walk_switch_body(&r#switch.body, scan);
            return;
        };
        let cases = r#switch.body.cases();
        let evaluated_case_count =
            start_index.map_or(cases.len(), |index| if cases[index].is_default() { cases.len() } else { index + 1 });
        for case in &cases[..evaluated_case_count] {
            if let Some(expression) = case.expression() {
                self.walk_expression(expression, scan);
            }
        }

        let Some(start_index) = start_index else {
            return;
        };

        let goto_statements = HashSet::new();
        let termination_cache = RefCell::new(HashMap::new());
        for case in &cases[start_index..] {
            for statement in case.statements() {
                self.walk_statement(statement, scan);
            }

            if switch_case_stops_before_fallthrough(
                case.statements(),
                scan.resolved_names,
                scan.classname,
                &goto_statements,
                &termination_cache,
            ) {
                break;
            }
        }
    }

    fn walk_try(&self, r#try: &'ast Try<'arena>, scan: &mut CallbackInvocationScan<'arena>) {
        let mut arms = Vec::new();
        let reachable_catches =
            reachable_catch_clauses(&r#try.block, r#try.catch_clauses.as_slice(), scan.resolved_names, scan.classname);
        for catch in &reachable_catches {
            if let Some(exception_types) = collect_caught_class_names(&catch.hint, scan.resolved_names, scan.classname)
            {
                let (rethrows, rethrows_types, rethrows_caught_types, rethrows_with_catches) = catch_escape_summary(
                    catch,
                    r#try.finally_clause.as_ref().map(|finally| &finally.block),
                    scan.resolved_names,
                    scan.classname,
                );
                arms.push(CallableExceptionCatchArm {
                    exception_types,
                    rethrows,
                    rethrows_types,
                    rethrows_caught_types,
                    rethrows_with_catches,
                });
            } else {
                scan.has_unresolved_catch_type = true;
            }
        }
        let previous_catch_groups = scan.catch_groups.len();
        if !arms.is_empty() {
            scan.catch_groups.push(CallableExceptionCatchGroup { arms, overrides_pending_exceptions: false });
        }
        self.walk_block(&r#try.block, scan);
        scan.catch_groups.truncate(previous_catch_groups);

        for catch in reachable_catches {
            self.walk_block(&catch.block, scan);
        }

        if let Some(finally) = &r#try.finally_clause {
            self.walk_block(&finally.block, scan);
        }
    }

    fn walk_function_call(&self, function_call: &'ast FunctionCall<'arena>, scan: &mut CallbackInvocationScan<'arena>) {
        let function = unparenthesized_expression(function_call.function);
        let parameter_name = normalize_variable_name(scan.parameter_name.as_bytes());
        let mut global_fallback_function = None;
        let invokes_parameter = match function {
            Expression::Variable(Variable::Direct(variable)) => {
                normalize_variable_name(variable.name) == parameter_name
            }
            Expression::Identifier(_) => {
                let Some((callback_parameter_offset, fallback_function)) =
                    immediate_builtin_callback_invocation(function, scan)
                else {
                    return mago_syntax::walker::walk_function_call(self, function_call, scan);
                };

                let invokes_parameter = function_call.argument_list.arguments.iter().enumerate().any(
                    |(offset, argument)| {
                        let is_callback_argument = match argument {
                            Argument::Positional(_) => offset == callback_parameter_offset,
                            Argument::Named(argument) => argument.name.value.eq_ignore_ascii_case(b"callback"),
                        };

                        is_callback_argument
                            && matches!(unparenthesized_expression(argument.value()), Expression::Variable(Variable::Direct(variable))
                                if normalize_variable_name(variable.name) == parameter_name)
                    },
                );
                global_fallback_function = invokes_parameter.then_some(fallback_function).flatten();

                invokes_parameter
            }
            _ => false,
        };

        if invokes_parameter {
            let invocation_offset = scan.invocation_catches.len();
            scan.invocation_catches.push(CallableExceptionCatchSummary {
                catch_groups: scan.catch_groups.iter().rev().cloned().collect(),
            });
            if let Some(function_name) = global_fallback_function {
                scan.invocation_catch_global_fallbacks.push((invocation_offset, function_name));
            }
            for function_name in &scan.inherited_global_fallback_functions {
                scan.invocation_catch_global_fallbacks.push((invocation_offset, *function_name));
            }
        }

        let synchronously_invoked_callback = immediate_builtin_callback_argument(function_call, scan);
        if let Some((callback_span, fallback_function)) = synchronously_invoked_callback {
            scan.synchronously_invoked_callbacks.push(callback_span);
            if let Some(function_name) = fallback_function {
                scan.inherited_global_fallback_functions.push(function_name);
            }
        }

        mago_syntax::walker::walk_function_call(self, function_call, scan);

        if let Some((callback_span, fallback_function)) = synchronously_invoked_callback {
            let _ = scan.synchronously_invoked_callbacks.pop();
            if fallback_function.is_some() {
                let _ = scan.inherited_global_fallback_functions.pop();
            }
            debug_assert!(
                !scan.synchronously_invoked_callbacks.contains(&callback_span),
                "nested callbacks must be popped in last-in, first-out order"
            );
        }
    }

    fn walk_function(&self, _: &'ast Function<'arena>, _: &mut CallbackInvocationScan<'arena>) {}

    fn walk_closure(&self, closure: &'ast Closure<'arena>, scan: &mut CallbackInvocationScan<'arena>) {
        let parameter_name = normalize_variable_name(scan.parameter_name.as_bytes());
        if !scan.synchronously_invoked_callbacks.contains(&closure.span())
            || closure
                .parameter_list
                .parameters
                .iter()
                .any(|parameter| normalize_variable_name(parameter.variable.name) == parameter_name)
            || !closure.use_clause.as_ref().is_some_and(|use_clause| {
                use_clause
                    .variables
                    .iter()
                    .any(|variable| normalize_variable_name(variable.variable.name) == parameter_name)
            })
        {
            return;
        }

        self.walk_block(&closure.body, scan);
    }

    fn walk_arrow_function(
        &self,
        arrow_function: &'ast ArrowFunction<'arena>,
        scan: &mut CallbackInvocationScan<'arena>,
    ) {
        let parameter_name = normalize_variable_name(scan.parameter_name.as_bytes());
        if !scan.synchronously_invoked_callbacks.contains(&arrow_function.span())
            || arrow_function
                .parameter_list
                .parameters
                .iter()
                .any(|parameter| normalize_variable_name(parameter.variable.name) == parameter_name)
        {
            return;
        }

        self.walk_expression(arrow_function.expression, scan);
    }

    fn walk_method(&self, _: &'ast Method<'arena>, _: &mut CallbackInvocationScan<'arena>) {}
}

impl CallbackInvocationWalker {
    fn walk_if_body_after_condition<'arena>(
        &self,
        body: &IfBody<'arena>,
        scan: &mut CallbackInvocationScan<'arena>,
        first_condition_truthiness: Option<bool>,
    ) {
        match body {
            IfBody::Statement(body) => {
                let mut can_reach_next_branch = first_condition_truthiness != Some(true);
                if first_condition_truthiness != Some(false) {
                    self.walk_statement(body.statement, scan);
                }

                for clause in &body.else_if_clauses {
                    if !can_reach_next_branch {
                        break;
                    }

                    self.walk_expression(clause.condition, scan);
                    match syntax_truthiness(clause.condition) {
                        Some(false) => {}
                        Some(true) => {
                            self.walk_statement(clause.statement, scan);
                            can_reach_next_branch = false;
                        }
                        None => {
                            self.walk_statement(clause.statement, scan);
                        }
                    }
                }

                if can_reach_next_branch && let Some(else_clause) = &body.else_clause {
                    self.walk_statement(else_clause.statement, scan);
                }
            }
            IfBody::ColonDelimited(body) => {
                let mut can_reach_next_branch = first_condition_truthiness != Some(true);
                if first_condition_truthiness != Some(false) {
                    for statement in &body.statements {
                        self.walk_statement(statement, scan);
                    }
                }

                for clause in &body.else_if_clauses {
                    if !can_reach_next_branch {
                        break;
                    }

                    self.walk_expression(clause.condition, scan);
                    match syntax_truthiness(clause.condition) {
                        Some(false) => {}
                        Some(true) => {
                            for statement in &clause.statements {
                                self.walk_statement(statement, scan);
                            }
                            can_reach_next_branch = false;
                        }
                        None => {
                            for statement in &clause.statements {
                                self.walk_statement(statement, scan);
                            }
                        }
                    }
                }

                if can_reach_next_branch && let Some(else_clause) = &body.else_clause {
                    for statement in &else_clause.statements {
                        self.walk_statement(statement, scan);
                    }
                }
            }
        }
    }
}

fn syntax_truthiness(expression: &Expression<'_>) -> Option<bool> {
    match unparenthesized_expression(expression) {
        Expression::Literal(Literal::True(_)) => Some(true),
        Expression::Literal(Literal::False(_) | Literal::Null(_)) => Some(false),
        Expression::Literal(Literal::Integer(value)) => value.value.map(|value| value != 0),
        Expression::Literal(Literal::Float(value)) => Some(value.value.0 != 0.0),
        Expression::Literal(Literal::String(value)) => value.value.map(|value| !value.is_empty() && value != b"0"),
        Expression::UnaryPrefix(unary) => match unary.operator {
            UnaryPrefixOperator::Not(_) => syntax_truthiness(unary.operand).map(|truthy| !truthy),
            UnaryPrefixOperator::BoolCast(_, _) | UnaryPrefixOperator::BooleanCast(_, _) => {
                syntax_truthiness(unary.operand)
            }
            _ => None,
        },
        _ => None,
    }
}

fn known_boolean_switch_start(r#switch: &Switch<'_>) -> Option<Option<usize>> {
    let switch_truthiness = match unparenthesized_expression(r#switch.expression) {
        Expression::Literal(Literal::True(_)) => true,
        Expression::Literal(Literal::False(_)) => false,
        _ => return None,
    };
    let mut default_index = None;

    for (index, case) in r#switch.body.cases().iter().enumerate() {
        let Some(expression) = case.expression() else {
            default_index = Some(index);
            continue;
        };

        if syntax_truthiness(expression)? == switch_truthiness {
            return Some(Some(index));
        }
    }

    Some(default_index)
}

fn immediate_builtin_callback_invocation(
    function: &Expression<'_>,
    scan: &CallbackInvocationScan<'_>,
) -> Option<(usize, Option<Word>)> {
    let Expression::Identifier(identifier) = unparenthesized_expression(function) else {
        return None;
    };
    let function_name = scan.resolved_names.resolve(identifier)?;
    let function_name = function_name.strip_prefix(b"\\").unwrap_or(function_name);
    let parameter_offset = immediate_builtin_callback_parameter(function_name, scan.php_version)?;
    let is_global_function = !function_name.contains(&b'\\');
    let is_namespaced_fallback = identifier.is_local() && !scan.resolved_names.is_imported(identifier);

    (is_global_function || is_namespaced_fallback).then(|| {
        let fallback_function = (is_namespaced_fallback && function_name.contains(&b'\\')).then(|| word(function_name));
        (parameter_offset, fallback_function)
    })
}

fn immediate_builtin_callback_argument(
    function_call: &FunctionCall<'_>,
    scan: &CallbackInvocationScan<'_>,
) -> Option<(Span, Option<Word>)> {
    let (callback_parameter_offset, fallback_function) =
        immediate_builtin_callback_invocation(function_call.function, scan)?;
    let callback = function_call.argument_list.arguments.iter().enumerate().find_map(|(offset, argument)| {
        let is_callback_argument = match argument {
            Argument::Positional(_) => offset == callback_parameter_offset,
            Argument::Named(argument) => argument.name.value.eq_ignore_ascii_case(b"callback"),
        };

        is_callback_argument.then(|| argument.value())
    })?;

    let callback = unparenthesized_expression(callback);
    matches!(callback, Expression::Closure(_) | Expression::ArrowFunction(_))
        .then_some((callback.span(), fallback_function))
}

fn immediate_builtin_callback_parameter(function_name: &[u8], php_version: PHPVersion) -> Option<usize> {
    let function_name = function_name.rsplit(|byte| *byte == b'\\').next()?;

    if php_version < PHPVersion::PHP84
        && [b"array_all".as_slice(), b"array_any", b"array_find", b"array_find_key"]
            .iter()
            .any(|name| function_name.eq_ignore_ascii_case(name))
    {
        return None;
    }

    [
        (b"array_all".as_slice(), 1),
        (b"array_any".as_slice(), 1),
        (b"array_filter".as_slice(), 1),
        (b"array_find".as_slice(), 1),
        (b"array_find_key".as_slice(), 1),
        (b"array_map".as_slice(), 0),
        (b"array_reduce".as_slice(), 1),
        (b"array_walk".as_slice(), 1),
        (b"array_walk_recursive".as_slice(), 1),
        (b"call_user_func".as_slice(), 0),
        (b"call_user_func_array".as_slice(), 0),
        (b"forward_static_call".as_slice(), 0),
        (b"forward_static_call_array".as_slice(), 0),
        (b"iterator_apply".as_slice(), 1),
        (b"preg_replace_callback".as_slice(), 1),
        (b"uasort".as_slice(), 1),
        (b"uksort".as_slice(), 1),
        (b"usort".as_slice(), 1),
    ]
    .into_iter()
    .find_map(|(name, offset)| function_name.eq_ignore_ascii_case(name).then_some(offset))
}

fn scan_callable_exception_catches<'arena>(
    metadata: &mut FunctionLikeMetadata,
    resolved_names: &'arena ResolvedNames<'arena>,
    classname: Option<Word>,
    php_version: PHPVersion,
    walk_body: impl Fn(&CallbackInvocationWalker, &mut CallbackInvocationScan<'arena>),
) {
    for parameter in &mut metadata.parameters {
        if parameter.callable_invocation_kind != Some(CallableInvocationKind::Immediately) {
            continue;
        }

        let mut scan = CallbackInvocationScan {
            parameter_name: parameter.get_name().0,
            resolved_names,
            classname,
            php_version,
            synchronously_invoked_callbacks: Vec::new(),
            inherited_global_fallback_functions: Vec::new(),
            catch_groups: Vec::new(),
            invocation_catches: Vec::new(),
            invocation_catch_global_fallbacks: Vec::new(),
            has_unresolved_catch_type: false,
        };
        walk_body(&CallbackInvocationWalker, &mut scan);
        if !scan.invocation_catches.is_empty() && !scan.has_unresolved_catch_type {
            parameter.callable_exception_catches = Some(scan.invocation_catches);
            parameter.callable_exception_catch_global_fallbacks = scan.invocation_catch_global_fallbacks;
        }
    }
}

fn collect_caught_class_names(
    hint: &Hint<'_>,
    resolved_names: &ResolvedNames<'_>,
    classname: Option<Word>,
) -> Option<Vec<Word>> {
    match hint {
        Hint::Identifier(identifier) => resolved_names.resolve(identifier).map(|name| vec![word(name)]),
        Hint::Self_(_) => classname.map(|classname| vec![classname]),
        Hint::Parenthesized(parenthesized) => collect_caught_class_names(parenthesized.hint, resolved_names, classname),
        Hint::Nullable(nullable) => collect_caught_class_names(nullable.hint, resolved_names, classname),
        Hint::Union(union) => {
            let left = collect_caught_class_names(union.left, resolved_names, classname)?;
            let right = collect_caught_class_names(union.right, resolved_names, classname)?;
            Some([left, right].concat())
        }
        Hint::Intersection(intersection) => {
            let left = collect_caught_class_names(intersection.left, resolved_names, classname)?;
            let right = collect_caught_class_names(intersection.right, resolved_names, classname)?;
            Some([left, right].concat())
        }
        _ => None,
    }
}

fn scan_function_like_docblock<A>(
    span: Span,
    functionlike_id: (Word, Word),
    metadata: &mut FunctionLikeMetadata,
    classname: Option<Word>,
    context: &Context<'_, '_, A>,
    scope: &NamespaceScope,
) where
    A: Arena,
{
    let Some(document) = parse_docblock(context, span) else {
        metadata.has_docblock = false;

        return;
    };

    metadata.has_docblock = true;

    for parse_error in document.errors {
        metadata.issues.push(
            Issue::error("Failed to parse function-like docblock comment.")
                .with_code(ScanningIssueKind::MalformedDocblockComment)
                .with_annotation(Annotation::primary(parse_error.span()).with_message(parse_error.to_string()))
                .with_note(parse_error.note())
                .with_help(parse_error.help()),
        );
    }

    let mut ignore_nullable_return = false;
    let mut ignore_falsable_return = false;

    for element in document.elements.iter() {
        let tag = match element {
            Element::Tag(tag) => tag,
            Element::Text(text) => {
                let has_inline_inherit_doc = text.segments.iter().any(|segment| {
                    matches!(
                        segment,
                        TextSegment::InlineTag(inline_tag)
                            if matches!(inline_tag.tag.value, TagValue::InheritDoc(_))
                    )
                });

                if has_inline_inherit_doc {
                    metadata.flags |= MetadataFlags::INHERITS_DOCS;
                }

                continue;
            }
            Element::Code(_) => continue,
        };

        if apply_function_like_metadata_flag(&mut metadata.flags, &mut metadata.explicit_docblock_flags, &tag.value) {
            continue;
        }

        match &tag.value {
            TagValue::IgnoreNullableReturn(_) => {
                metadata.flags |= MetadataFlags::IGNORE_NULLABLE_RETURN;
                ignore_nullable_return = true;
            }
            TagValue::IgnoreFalsableReturn(_) => {
                metadata.flags |= MetadataFlags::IGNORE_FALSABLE_RETURN;
                ignore_falsable_return = true;
            }
            TagValue::InheritDoc(_) => {
                metadata.flags |= MetadataFlags::INHERITS_DOCS;
            }
            _ => {}
        }
    }

    let mut type_context = metadata.type_resolution_context.take().unwrap_or_default();
    for tag in document.tags() {
        let TagValue::Template(template) = &tag.value else {
            continue;
        };

        let template_name = word(template.name.value);
        let template_as_type = if let Some(bound) = &template.bound {
            match builder::get_union_from_type(bound.r#type, scope, &type_context, classname) {
                Ok(tunion) => tunion,
                Err(typing_error) => {
                    metadata.issues.push(typing_error_issue(
                        "Invalid `@template` type.",
                        ScanningIssueKind::InvalidTemplateTag,
                        &typing_error,
                    ));

                    continue;
                }
            }
        } else {
            get_mixed()
        };

        let template_default = if let Some(default) = &template.default {
            match builder::get_union_from_type(default.r#type, scope, &type_context, classname) {
                Ok(tunion) => Some(tunion),
                Err(typing_error) => {
                    metadata.issues.push(typing_error_issue(
                        "Invalid `@template` default type.",
                        ScanningIssueKind::InvalidTemplateTag,
                        &typing_error,
                    ));

                    None
                }
            }
        } else {
            None
        };

        let definition = GenericTemplate::new(GenericParent::FunctionLike(functionlike_id), template_as_type)
            .with_default(template_default);

        metadata.add_template_type(template_name, definition.clone());
        type_context = type_context.with_template_definition(template_name, vec![definition]);
    }

    // Names of the declared parameters, used to tell Psalm-style parameter links
    // (`@param $prototype $value`, `@return $value`) apart from typos (`@return $mixed`).
    let parameter_names: Vec<Word> = metadata.parameters.iter().map(|parameter| parameter.get_name().0).collect();

    for_each_tag_by_ascending_trust(&document, |tag| {
        let TagValue::Param(parameter_tag) = &tag.value else {
            return;
        };

        let Some(parameter_variable) = parameter_tag.parameter else {
            return;
        };

        let parameter_name = word(parameter_variable.value);
        let is_variadic = parameter_tag.is_variadic();

        let Some(parameter_metadata) = metadata.get_parameter_mut(parameter_name) else {
            metadata.issues.push(
                Issue::error("The @param tag references an unknown parameter.")
                    .with_code(ScanningIssueKind::InvalidParamTag)
                    .with_annotation(
                        Annotation::primary(parameter_tag.span())
                            .with_message(format!("Parameter `{parameter_variable}` is not defined in this function")),
                    )
                    .with_note(
                        "Each `@param` tag in a docblock must correspond to a parameter in the function's signature.",
                    )
                    .with_help("Please check for typos or add the parameter to the function signature."),
            );

            return;
        };

        let mut variadic_mismatch_issue = None;
        if is_variadic && !parameter_metadata.flags.is_variadic() {
            let parameter_span = parameter_metadata.get_span();
            parameter_metadata.flags |= MetadataFlags::VARIADIC;

            variadic_mismatch_issue = Some(
                Issue::error("@param tag has a variadic mismatch.")
                    .with_code(ScanningIssueKind::InvalidParamTag)
                    .with_annotation(Annotation::primary(parameter_tag.span()).with_message(
                        "This docblock declares the parameter as variadic, but the function signature does not",
                    ))
                    .with_annotation(
                        Annotation::secondary(parameter_span)
                            .with_message("The parameter is declared here without being variadic"),
                    )
                    .with_note("The use of `...` in the `@param` tag must match the function's parameter declaration.")
                    .with_help("Either add `...` to the parameter in the function signature or remove it from the `@param` tag."),
            );
        }

        match reject_unresolvable_tag_variable(parameter_tag.r#type, |name| parameter_names.contains(&word(name)))
            .and_then(|()| get_type_metadata_from_type(parameter_tag.r#type, classname, &type_context, scope))
        {
            Ok(mut provided_type) => {
                let resulting_type = if !is_variadic
                    && parameter_metadata.flags.is_variadic()
                    && let Some(array_value) = provided_type.type_union.get_single_value_of_array_like()
                {
                    provided_type.type_union = array_value.into_owned();
                    provided_type
                } else {
                    provided_type
                };

                let real_type = parameter_metadata.type_metadata.as_ref();
                let resulting_type = merge_type_preserving_nullability(resulting_type, real_type);

                parameter_metadata.set_type_metadata(Some(resulting_type));
            }
            Err(typing_error) => {
                metadata.issues.push(typing_error_issue(
                    "Could not resolve the type for the @param tag.",
                    ScanningIssueKind::InvalidParamTag,
                    &typing_error,
                ));
            }
        }

        if let Some(variadic_mismatch_issue) = variadic_mismatch_issue {
            metadata.issues.push(variadic_mismatch_issue);
        }
    });

    for tag in document.tags() {
        let (parameter, invocation_kind) = match &tag.value {
            TagValue::ParamImmediatelyInvokedCallable(tag) => (tag.parameter, CallableInvocationKind::Immediately),
            TagValue::ParamLaterInvokedCallable(tag) => (tag.parameter, CallableInvocationKind::Later),
            _ => continue,
        };

        let parameter_name = word(parameter.value);
        let Some(parameter_metadata) = metadata.get_parameter_mut(parameter_name) else {
            metadata.issues.push(
                Issue::error("Callable invocation tag references an unknown parameter.")
                    .with_code(ScanningIssueKind::InvalidParamTag)
                    .with_annotation(
                        Annotation::primary(tag.span())
                            .with_message(format!("Parameter `{parameter}` is not defined in this function")),
                    )
                    .with_note("Callable invocation tags must reference a parameter in the function signature.")
                    .with_help("Check for typos or add the parameter to the function signature."),
            );

            continue;
        };

        parameter_metadata.callable_invocation_kind = Some(invocation_kind);
    }

    for parameter_metadata in metadata.get_parameters_mut() {
        parameter_metadata.add_null_from_null_default();
    }

    for tag in document.tags() {
        let TagValue::ParamOut(param_out) = &tag.value else {
            continue;
        };

        let param_name = word(param_out.parameter.value);

        let Some(parameter_metadata) = metadata.get_parameter_mut(param_name) else {
            metadata.issues.push(
                Issue::error("@param-out tag references an unknown parameter.")
                    .with_code(ScanningIssueKind::InvalidParamOutTag)
                    .with_annotation(
                        Annotation::primary(param_out.span())
                            .with_message(format!("Parameter `{}` does not exist", param_out.parameter)),
                    )
                    .with_note("The `@param-out` tag specifies the type of a by-reference parameter after the function has executed.")
                    .with_help("Check for typos or ensure this parameter exists in the function signature."),
            );

            continue;
        };

        if !parameter_metadata.flags.is_by_reference() {
            metadata.issues.push(
                Issue::error("@param-out tag used on a non-by-reference parameter")
                    .with_code(ScanningIssueKind::InvalidParamOutTag)
                    .with_annotation(
                        Annotation::primary(param_out.span())
                            .with_message("This parameter is not declared as by-reference"),
                    )
                    .with_note("The `@param-out` tag can only be used with parameters that are passed by reference.")
                    .with_help("Ensure the parameter is declared with `&` in the function signature."),
            );

            continue;
        }

        match get_type_metadata_from_type(param_out.r#type, classname, &type_context, scope) {
            Ok(parameter_out_type) => {
                parameter_metadata.out_type = Some(parameter_out_type);
            }
            Err(typing_error) => {
                metadata.issues.push(typing_error_issue(
                    "Invalid `@param-out` type string.",
                    ScanningIssueKind::InvalidParamOutTag,
                    &typing_error,
                ));
            }
        }
    }

    for tag in document.tags() {
        let TagValue::ParamClosureThis(param_closure_this) = &tag.value else {
            continue;
        };

        let param_name = word(param_closure_this.parameter.value);

        let Some(parameter_metadata) = metadata.get_parameter_mut(param_name) else {
            metadata.issues.push(
                Issue::error("@param-closure-this tag references an unknown parameter.")
                    .with_code(ScanningIssueKind::InvalidParamClosureThisTag)
                    .with_annotation(
                        Annotation::primary(param_closure_this.span())
                            .with_message(format!("Parameter `{}` does not exist", param_closure_this.parameter)),
                    )
                    .with_note("The `@param-closure-this` tag specifies the type of `$this` bound inside a closure passed as this parameter.")
                    .with_help("Check for typos or ensure this parameter exists in the function signature."),
            );

            continue;
        };

        match get_type_metadata_from_type(param_closure_this.r#type, classname, &type_context, scope) {
            Ok(closure_this_type) => {
                parameter_metadata.closure_this_type = Some(closure_this_type);
            }
            Err(typing_error) => {
                metadata.issues.push(typing_error_issue(
                    "Invalid `@param-closure-this` type string.",
                    ScanningIssueKind::InvalidParamClosureThisTag,
                    &typing_error,
                ));
            }
        }
    }

    let return_tag = find_most_trusted_tag(&document, |tag| match &tag.value {
        TagValue::Return(return_tag) => Some(*return_tag),
        _ => None,
    });

    if let Some(return_type) = return_tag.as_ref() {
        match reject_unresolvable_tag_variable(return_type.r#type, |name| parameter_names.contains(&word(name)))
            .and_then(|()| get_type_metadata_from_type(return_type.r#type, classname, &type_context, scope))
        {
            Ok(return_type_signature) => {
                let is_abstract_method = metadata.method_metadata.as_ref().is_some_and(|m| m.is_abstract);
                let return_type_signature = if is_abstract_method {
                    return_type_signature
                } else {
                    let real_type = metadata.return_type_declaration_metadata.as_ref();
                    merge_type_preserving_nullability(return_type_signature, real_type)
                };
                metadata.set_return_type_metadata(Some(return_type_signature));
            }
            Err(typing_error) => {
                metadata.issues.push(typing_error_issue(
                    "Failed to resolve `@return` type string.",
                    ScanningIssueKind::InvalidReturnTag,
                    &typing_error,
                ));
            }
        }
    }

    for tag in document.tags() {
        let TagValue::Where(where_tag) = &tag.value else {
            continue;
        };

        let Some(method_metadata) = metadata.get_method_metadata_mut() else {
            metadata.issues.push(
                Issue::error("`@where` tag cannot be used on functions or closures.")
                    .with_code(ScanningIssueKind::InvalidWhereTag)
                    .with_annotation(
                        Annotation::primary(where_tag.span())
                            .with_message("`@where` is only valid on instance methods"),
                    )
                    .with_note("The `@where` tag constrains template types based on the instance type of `$this`. Functions and closures do not have a `$this` context.")
                    .with_help("Remove the `@where` tag. If you need this logic, consider refactoring it into an instance method on a class."),
            );

            continue;
        };

        if method_metadata.is_static {
            metadata.issues.push(
                Issue::error("`@where` tag cannot be used on static methods.")
                    .with_code(ScanningIssueKind::InvalidWhereTag)
                    .with_annotation(
                        Annotation::primary(where_tag.span())
                            .with_message("This constraint is not allowed on a static method"),
                    )
                    .with_note("The `@where` tag constrains template types based on the instance type of `$this`. Static methods are not tied to an instance and have no `$this` context.")
                    .with_help("Remove the `@where` tag. To constrain a template type on a static method, use a type bound like `@template T of SomeInterface` instead."),
            );

            continue;
        }

        match get_type_metadata_from_type(where_tag.r#type, classname, &type_context, scope) {
            Ok(constraint_type) => {
                let template_name = word(where_tag.name.value);

                method_metadata.where_constraints.insert(template_name, constraint_type);
            }
            Err(typing_error) => metadata.issues.push(
                Issue::error(format!("Invalid constraint type `{}` in `@where` tag.", where_tag.r#type))
                    .with_code(ScanningIssueKind::InvalidWhereTag)
                    .with_annotation(Annotation::primary(typing_error.span()).with_message(typing_error.to_string()))
                    .with_note(typing_error.note())
                    .with_help(typing_error.help()),
            ),
        }
    }

    for tag in document.tags() {
        let TagValue::Throws(thrown) = &tag.value else {
            continue;
        };

        match get_type_metadata_from_type(thrown.r#type, classname, &type_context, scope) {
            Ok(thrown_type) => {
                metadata.thrown_types.push(thrown_type);
            }
            Err(typing_error) => {
                metadata.issues.push(typing_error_issue(
                    "Invalid `@throws` type string.",
                    ScanningIssueKind::InvalidThrowsTag,
                    &typing_error,
                ));
            }
        }
    }

    for tag in document.tags() {
        let (TagValue::Assert(assertion_tag)
        | TagValue::AssertIfTrue(assertion_tag)
        | TagValue::AssertIfFalse(assertion_tag)) = &tag.value
        else {
            continue;
        };

        let assertion_subject = assertion_subject_word(&assertion_tag.subject);
        let assertions = parse_assertions_from_tag(assertion_tag, classname, &type_context, scope, metadata);

        let bucket = match &tag.value {
            TagValue::AssertIfTrue(_) => &mut metadata.if_true_assertions,
            TagValue::AssertIfFalse(_) => &mut metadata.if_false_assertions,
            _ => &mut metadata.assertions,
        };

        for assertion in assertions {
            bucket.entry(assertion_subject).or_default().push(assertion);
        }
    }

    metadata.type_resolution_context = Some(type_context);

    if ignore_nullable_return || ignore_falsable_return {
        if let Some(return_type) = &mut metadata.return_type_metadata {
            return_type.type_union.set_ignore_nullable_issues(ignore_nullable_return);
            return_type.type_union.set_ignore_falsable_issues(ignore_falsable_return);
        }

        if let Some(return_type) = &mut metadata.return_type_declaration_metadata {
            return_type.type_union.set_ignore_nullable_issues(ignore_nullable_return);
            return_type.type_union.set_ignore_falsable_issues(ignore_falsable_return);
        }
    }
}

fn parse_assertions_from_tag(
    assertion_tag: &AssertTagValue<'_>,
    classname: Option<Word>,
    type_context: &TypeResolutionContext,
    scope: &NamespaceScope,
    function_like_metadata: &mut FunctionLikeMetadata,
) -> Vec<Assertion> {
    let mut assertions = Vec::new();

    let is_negation = assertion_tag.is_negated();
    let is_equal = assertion_tag.is_equality();

    let asserted_type = match &assertion_tag.pattern {
        AssertPattern::Truthy(_) => {
            assertions.push(if is_negation { Assertion::Falsy } else { Assertion::Truthy });

            return assertions;
        }
        AssertPattern::Falsy(_) => {
            assertions.push(if is_negation { Assertion::Truthy } else { Assertion::Falsy });

            return assertions;
        }
        AssertPattern::NonEmpty(_) => {
            assertions.push(if is_negation { Assertion::Empty } else { Assertion::NonEmpty });

            return assertions;
        }
        AssertPattern::Type(asserted_type) => asserted_type,
    };

    if !is_equal && matches!(asserted_type, Type::Empty(_)) {
        assertions.push(if is_negation { Assertion::NonEmpty } else { Assertion::Empty });

        return assertions;
    }

    match get_type_metadata_from_type(asserted_type, classname, type_context, scope) {
        Ok(type_metadata) => match (is_equal, is_negation) {
            (true, true) => {
                for atomic in type_metadata.type_union.types.into_owned() {
                    assertions.push(Assertion::IsNotIdentical(atomic));
                }
            }
            (true, false) => {
                for atomic in type_metadata.type_union.types.into_owned() {
                    assertions.push(Assertion::IsIdentical(atomic));
                }
            }
            (false, true) => {
                for atomic in type_metadata.type_union.types.into_owned() {
                    assertions.push(Assertion::IsNotType(atomic));
                }
            }
            (false, false) => {
                for atomic in type_metadata.type_union.types.into_owned() {
                    assertions.push(Assertion::IsType(atomic));
                }
            }
        },
        Err(typing_error) => {
            function_like_metadata.issues.push(typing_error_issue(
                "Failed to resolve assertion type string.",
                ScanningIssueKind::InvalidAssertionTag,
                &typing_error,
            ));
        }
    }

    assertions
}

/// Collects every variable imported via `global $x;` anywhere in `block`, without
/// descending into nested function/closure/arrow-function definitions (those are
/// separate scopes).
pub fn collect_globals_into(block: &Block, globals: &mut WordSet) {
    for statement in &block.statements {
        collect_globals_from_statement(statement, globals);
    }
}

fn collect_globals_from_statement(statement: &Statement, globals: &mut WordSet) {
    match statement {
        Statement::Global(global) => {
            for variable in &global.variables {
                if let Variable::Direct(direct) = variable {
                    globals.insert(word(direct.name));
                }
            }
        }
        Statement::Block(block) => collect_globals_into(block, globals),
        Statement::Namespace(namespace) => {
            for statement in namespace.statements() {
                collect_globals_from_statement(statement, globals);
            }
        }
        Statement::If(r#if) => match &r#if.body {
            IfBody::Statement(body) => {
                collect_globals_from_statement(body.statement, globals);
                for else_if in &body.else_if_clauses {
                    collect_globals_from_statement(else_if.statement, globals);
                }
                if let Some(else_clause) = &body.else_clause {
                    collect_globals_from_statement(else_clause.statement, globals);
                }
            }
            IfBody::ColonDelimited(body) => {
                for statement in &body.statements {
                    collect_globals_from_statement(statement, globals);
                }
                for else_if in &body.else_if_clauses {
                    for statement in &else_if.statements {
                        collect_globals_from_statement(statement, globals);
                    }
                }
                if let Some(else_clause) = &body.else_clause {
                    for statement in &else_clause.statements {
                        collect_globals_from_statement(statement, globals);
                    }
                }
            }
        },
        Statement::For(r#for) => match &r#for.body {
            ForBody::Statement(statement) => collect_globals_from_statement(statement, globals),
            ForBody::ColonDelimited(body) => {
                for statement in &body.statements {
                    collect_globals_from_statement(statement, globals);
                }
            }
        },
        Statement::Foreach(foreach) => match &foreach.body {
            ForeachBody::Statement(statement) => collect_globals_from_statement(statement, globals),
            ForeachBody::ColonDelimited(body) => {
                for statement in &body.statements {
                    collect_globals_from_statement(statement, globals);
                }
            }
        },
        Statement::While(r#while) => match &r#while.body {
            WhileBody::Statement(statement) => collect_globals_from_statement(statement, globals),
            WhileBody::ColonDelimited(body) => {
                for statement in &body.statements {
                    collect_globals_from_statement(statement, globals);
                }
            }
        },
        Statement::DoWhile(do_while) => collect_globals_from_statement(do_while.statement, globals),
        Statement::Switch(switch) => {
            let cases = match &switch.body {
                SwitchBody::BraceDelimited(body) => &body.cases,
                SwitchBody::ColonDelimited(body) => &body.cases,
            };
            for case in cases {
                match case {
                    SwitchCase::Expression(case) => {
                        for statement in &case.statements {
                            collect_globals_from_statement(statement, globals);
                        }
                    }
                    SwitchCase::Default(case) => {
                        for statement in &case.statements {
                            collect_globals_from_statement(statement, globals);
                        }
                    }
                }
            }
        }
        Statement::Try(r#try) => {
            for statement in &r#try.block.statements {
                collect_globals_from_statement(statement, globals);
            }
            for catch in &r#try.catch_clauses {
                for statement in &catch.block.statements {
                    collect_globals_from_statement(statement, globals);
                }
            }
            if let Some(finally) = &r#try.finally_clause {
                for statement in &finally.block.statements {
                    collect_globals_from_statement(statement, globals);
                }
            }
        }
        _ => {}
    }
}
