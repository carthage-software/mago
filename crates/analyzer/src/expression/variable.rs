use std::collections::BTreeMap;
use std::rc::Rc;

use mago_codex::data_flow::graph::DataFlowGraph;
use mago_codex::data_flow::graph::GraphKind;
use mago_codex::data_flow::node::DataFlowNode;
use mago_codex::data_flow::node::DataFlowNodeId;
use mago_codex::data_flow::node::DataFlowNodeKind;
use mago_codex::data_flow::path::PathKind;
use mago_codex::misc::VariableIdentifier;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::get_bool;
use mago_codex::ttype::get_float;
use mago_codex::ttype::get_int_range;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::get_non_empty_string;
use mago_codex::ttype::get_null;
use mago_codex::ttype::get_string;
use mago_codex::ttype::union::TUnion;
use mago_interner::StringIdentifier;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::issue::TypingIssueKind;

use super::assignment;

impl Analyzable for Variable {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        match self {
            Variable::Direct(var) => var.analyze(context, block_context, artifacts),
            Variable::Indirect(var) => var.analyze(context, block_context, artifacts),
            Variable::Nested(var) => var.analyze(context, block_context, artifacts),
        }
    }
}

impl Analyzable for DirectVariable {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let name = context.interner.lookup(&self.name);
        let resulting_type = read_variable(context, block_context, artifacts, true, name, self.span());

        artifacts.set_expression_type(self, resulting_type);

        Ok(())
    }
}

impl Analyzable for IndirectVariable {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        self.expression.analyze(context, block_context, artifacts)?;

        let resulting_type = match artifacts.get_expression_type(&self.expression) {
            Some(expression_type) if expression_type.is_single() => {
                match expression_type.get_single_literal_string_value() {
                    Some(value) => {
                        let variable_name = format!("${value}");

                        read_variable(context, block_context, artifacts, false, &variable_name, self.span())
                    }
                    _ => get_mixed_any(),
                }
            }
            _ => get_mixed_any(),
        };

        artifacts.set_expression_type(self, resulting_type);

        Ok(())
    }
}

impl Analyzable for NestedVariable {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        self.variable.analyze(context, block_context, artifacts)?;

        let resulting_type = match artifacts.get_expression_type(&self.variable) {
            Some(expression_type) if expression_type.is_single() => {
                match expression_type.get_single_literal_string_value() {
                    Some(value) => {
                        let variable_name = format!("${value}");

                        read_variable(context, block_context, artifacts, false, &variable_name, self.span())
                    }
                    _ => get_mixed_any(),
                }
            }
            _ => get_mixed_any(),
        };

        artifacts.set_expression_type(self, resulting_type);

        Ok(())
    }
}

fn read_variable<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    is_direct_variable: bool,
    variable_name: &str,
    variable_span: Span,
) -> TUnion {
    let _ = block_context.has_variable(variable_name);

    let variable_type = match block_context.locals.get(variable_name) {
        Some(variable_type) => (**variable_type).clone(),
        None => {
            if let Some(global_variable_type) = get_global_variable_type(variable_name) {
                block_context.locals.insert(variable_name.to_string(), Rc::new(global_variable_type.clone()));

                global_variable_type
            } else if block_context.variables_possibly_in_scope.contains(variable_name) {
                let mut issue =  Issue::warning(format!(
                    "Variable `{variable_name}` might not have been defined on all execution paths leading to this point.",
                ))
                .with_annotation(
                    Annotation::primary(variable_span)
                        .with_message(format!("`{variable_name}` might be undefined here")),
                )
                .with_note("This can happen if the variable is assigned within a conditional block and there's an execution path to this usage where that block is skipped.")
                .with_note("Accessing an undefined variable will result in an `E_WARNING` (PHP 8+) or `E_NOTICE` (PHP 7) and it will be treated as `null`.")
                .with_help(format!("Initialize `{variable_name}` before conditional paths, or use `isset()` to check its existence."));

                if let Some(most_recent_assignment_span) =
                    artifacts.data_flow_graph.get_variable_most_recent_assignment_span(VariableIdentifier(
                        context.interner.intern(variable_name),
                    ))
                {
                    issue =
                        issue
                            .with_annotation(Annotation::secondary(most_recent_assignment_span).with_message(format!(
                            "`{variable_name}` assigned here, but this path might not always execute before its use."
                        )));
                }

                context.buffer.report(TypingIssueKind::PossiblyUndefinedVariable, issue);

                get_mixed()
            } else if block_context.inside_variable_reference {
                context.buffer.report(
                    TypingIssueKind::ReferenceToUndefinedVariable,
                    Issue::help(format!("Reference created from a previously undefined variable `{variable_name}`.",))
                        .with_annotation(
                            Annotation::primary(variable_span)
                                .with_message(format!("`{variable_name}` is created here and initialized to `null` because it's used as a reference")),
                        )
                        .with_note(
                            "When a reference is taken from an undefined variable, PHP creates it with a `null` value."
                        )
                        .with_note(
                            "This is often used for output parameters but can hide typos if you intended to use an existing variable."
                        )
                        .with_help(
                            format!("If this is intentional, consider initializing `{variable_name}` to `null` first for code clarity. Otherwise, check for typos.")
                        ),
                );

                // This variable does not currently exist, but is being referenced.
                // therefore, we need to analyze it as if it was being assigned `null`.
                assignment::analyze_assignment_to_variable(
                    context,
                    block_context,
                    artifacts,
                    is_direct_variable,
                    variable_span,
                    None,
                    get_null(),
                    variable_name,
                    false,
                );

                get_null()
            } else if block_context.inside_unset {
                get_null()
            } else {
                let mut issue = Issue::error(format!("Undefined variable: `{variable_name}`.")).with_annotation(
                    Annotation::primary(variable_span)
                        .with_message(format!("Variable `{variable_name}` used here but not defined")),
                );

                let mut has_confusable_characters = false;
                if let Some(confusable_note) = generate_confusable_character_note(variable_name) {
                    has_confusable_characters = true;
                    issue = issue.with_note(confusable_note);
                }

                let similar_suggestions = find_similar_variable_names(block_context, variable_name);

                let mut help_message =
                    format!("Ensure `{variable_name}` is assigned a value before this use, or check its scope.");
                if !similar_suggestions.is_empty() {
                    let suggestions_str = similar_suggestions.join("`, `");
                    issue = issue.with_note(format!(
                        "Did you perhaps mean one of these defined variables: `{suggestions_str}`?"
                    ));

                    help_message = format!(
                        "Check for typos (like those suggested above), ensure `{variable_name}` is assigned, or verify its scope."
                    );
                } else if !has_confusable_characters {
                    // Only add generic typo help if no confusable chars and no specific suggestions.
                    help_message = format!(
                        "Ensure `{variable_name}` is assigned before use, or check for typos and variable scope."
                    );
                }

                context.buffer.report(TypingIssueKind::UndefinedVariable, issue.with_help(help_message));

                get_mixed_any()
            }
        }
    };

    if variable_type.possibly_undefined_from_try {
        context.buffer.report(
            TypingIssueKind::PossiblyUndefinedVariable,
            Issue::warning(format!(
                "Variable `{variable_name}` might be undefined here because its assignment occurs within a `try` block.",
            ))
            .with_annotation(
                Annotation::primary(variable_span)
                    .with_message(format!("`{variable_name}` might be undefined due to an exception in the preceding `try` block")),
            )
            .with_note(
                "This variable is assigned inside a `try` block. If an exception was thrown before this assignment was reached, the variable would not be defined in this context."
            )
            .with_note(
                "Accessing an undefined variable will result in an `E_WARNING` (PHP 8+) or `E_NOTICE` (PHP 7) and it will be treated as `null`."
            )
            .with_help(format!(
                "Initialize `{variable_name}` before the `try` block if it should always exist, or use `isset()` to check its existence.",
            )),
        );
    }

    check_for_shadowed_loop_variable(variable_name, &variable_type, context, block_context, artifacts);

    add_dataflow_to_variable(
        (context.interner.intern(variable_name), variable_span),
        variable_type,
        block_context,
        artifacts,
    )
}

fn check_for_shadowed_loop_variable(
    name: &str,
    variable_type: &TUnion,
    context: &mut Context<'_>,
    block_context: &BlockContext,
    artifacts: &AnalysisArtifacts,
) {
    if variable_type.parent_nodes.len() <= 1
        || block_context.inside_loop_expressions
        || block_context.for_loop_init_bounds.0 != 0
        || block_context.inside_assignment_operation
        || artifacts.data_flow_graph.kind != GraphKind::FunctionBody
    {
        return;
    }

    let mut loop_init_span: Option<Span> = None;

    for parent_node in &variable_type.parent_nodes {
        if let DataFlowNodeKind::VariableUseSource { span: for_loop_init_pos, from_loop_init: true, .. } =
            parent_node.kind
        {
            if let Some(loop_init_span_inner) = loop_init_span {
                if for_loop_init_pos.start.offset < loop_init_span_inner.start.offset {
                    loop_init_span = Some(for_loop_init_pos);
                }
            } else {
                loop_init_span = Some(for_loop_init_pos);
            }
        }
    }

    if let Some(loop_init_pos) = loop_init_span {
        for parent_node in &variable_type.parent_nodes {
            if let DataFlowNodeKind::VariableUseSource {
                has_parent_nodes: true,
                from_loop_init: false,
                span: parent_node_pos,
                ..
            } = parent_node.kind
                && parent_node_pos.start.offset < loop_init_pos.start.offset
            {
                context.buffer.report(
                        TypingIssueKind::ShadowedLoopVar,
                        Issue::error(format!(
                            "Assignment to `{name}` within this loop construct shadows an outer variable used after the loop."
                        ))
                        .with_annotation(
                            Annotation::primary(loop_init_pos)
                                .with_message(format!("`{name}` assigned here shadows the outer variable."))
                        )
                        .with_note(
                            "The original value of the variable defined before the loop will be overwritten before it is used later."
                        )
                        .with_help(
                            format!(
                                "Rename the loop variable or the outer variable `{name}` to avoid this shadowing."
                            )
                        ),
                    );

                break;
            }
        }
    }
}

fn add_dataflow_to_variable(
    variable: (StringIdentifier, Span),
    mut variable_type: TUnion,
    block_context: &mut BlockContext,
    artifacts: &mut AnalysisArtifacts,
) -> TUnion {
    let data_flow_graph = &mut artifacts.data_flow_graph;
    let GraphKind::FunctionBody = data_flow_graph.kind else {
        return variable_type;
    };

    if !block_context.inside_general_use
        && !block_context.inside_call
        && !block_context.inside_throw
        && !block_context.inside_isset
        && !block_context.inside_conditional
        && !block_context.inside_return
        && !block_context.inside_loop_expressions
    {
        return variable_type;
    }

    add_dataflow_to_used_variable(variable, data_flow_graph, &mut variable_type);

    variable_type
}

fn add_dataflow_to_used_variable(
    variable: (StringIdentifier, Span),
    data_flow_graph: &mut DataFlowGraph,
    variable_type: &mut TUnion,
) {
    let sink_node = DataFlowNode {
        id: DataFlowNodeId::Var(VariableIdentifier(variable.0), variable.1),
        kind: DataFlowNodeKind::VariableUseSink { span: variable.1 },
    };

    data_flow_graph.add_node(sink_node.clone());

    let mut parent_nodes = variable_type.parent_nodes.clone();

    if parent_nodes.is_empty() {
        parent_nodes.push(sink_node);
    } else {
        for parent_node in &parent_nodes {
            data_flow_graph.add_path(parent_node, &sink_node, PathKind::Default);
        }
    }

    variable_type.parent_nodes = parent_nodes;
}

fn get_global_variable_type(variable_name: &str) -> Option<TUnion> {
    Some(match variable_name {
        "$argv" => {
            let mut argv = TUnion::new(vec![
                TAtomic::Null,
                TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![TAtomic::Scalar(
                    TScalar::string(),
                )]))))),
            ]);
            argv.ignore_nullable_issues = true;

            argv
        }
        "$argc" => {
            let mut argc = TUnion::new(vec![TAtomic::Null, TAtomic::Scalar(TScalar::Integer(TInteger::From(1)))]);

            argc.ignore_nullable_issues = true;
            argc
        }
        "$http_response_header" => {
            let mut http_response_header =
                TUnion::new(vec![TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                    TAtomic::Scalar(TScalar::String(TString::general_with_props(false, true, true))),
                ])))))]);

            http_response_header.possibly_undefined = true; // undefined in cli
            http_response_header
        }
        "$GLOBALS" => {
            let mut known_items = BTreeMap::new();
            known_items.insert(
                ArrayKey::String("arvc".to_owned()),
                (true, TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::From(1)))])),
            );
            known_items.insert(
                ArrayKey::String("argv".to_owned()),
                (
                    true,
                    TUnion::new(vec![TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                        TAtomic::Scalar(TScalar::string()),
                    ])))))]),
                ),
            );

            TUnion::new(vec![TAtomic::Array(TArray::Keyed(TKeyedArray {
                known_items: Some(known_items),
                parameters: Some((
                    Box::new(TUnion::new(vec![TAtomic::Scalar(TScalar::non_empty_string())])),
                    Box::new(get_mixed()),
                )),
                non_empty: true,
            }))])
        }
        "$_COOKIE" => TUnion::new(vec![TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
            Box::new(TUnion::new(vec![TAtomic::Scalar(TScalar::non_empty_string())])),
            Box::new(TUnion::new(vec![TAtomic::Scalar(TScalar::string())])),
        )))]),
        "$_GET" | "$_POST" | "$_REQUEST" => {
            TUnion::new(vec![TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
                Box::new(TUnion::new(vec![
                    TAtomic::Scalar(TScalar::non_empty_string()),
                    TAtomic::Scalar(TScalar::int()),
                ])),
                Box::new(TUnion::new(vec![
                    TAtomic::Scalar(TScalar::string()),
                    TAtomic::Array(TArray::Keyed(
                        TKeyedArray::new_with_parameters(
                            Box::new(TUnion::new(vec![
                                TAtomic::Scalar(TScalar::non_empty_string()),
                                TAtomic::Scalar(TScalar::int()),
                            ])),
                            Box::new(TUnion::new(vec![
                                TAtomic::Scalar(TScalar::string()),
                                TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
                                    Box::new(TUnion::new(vec![
                                        TAtomic::Scalar(TScalar::non_empty_string()),
                                        TAtomic::Scalar(TScalar::int()),
                                    ])),
                                    Box::new(get_mixed_any()),
                                ))),
                            ])),
                        )
                        .to_non_empty(),
                    )),
                ])),
            )))])
        }
        "$_SERVER" => {
            let time =
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;

            let mut known_items = BTreeMap::new();
            // Standard CGI/1.1 and PHP variables
            known_items.insert(ArrayKey::String("PHP_SELF".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("GATEWAY_INTERFACE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SERVER_ADDR".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SERVER_NAME".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SERVER_SOFTWARE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SERVER_PROTOCOL".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("REQUEST_METHOD".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("REQUEST_TIME".to_owned()), (true, get_int_range(Some(time), None)));
            known_items.insert(ArrayKey::String("REQUEST_TIME_FLOAT".to_owned()), (true, get_float()));
            known_items.insert(ArrayKey::String("QUERY_STRING".to_owned()), (true, get_string()));
            known_items.insert(ArrayKey::String("DOCUMENT_ROOT".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_ACCEPT".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_ACCEPT_CHARSET".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_ACCEPT_ENCODING".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_ACCEPT_LANGUAGE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_CONNECTION".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_HOST".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_REFERER".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_USER_AGENT".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTPS".to_owned()), (true, get_string()));
            known_items.insert(ArrayKey::String("REMOTE_ADDR".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("REMOTE_HOST".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("REMOTE_PORT".to_owned()), (true, get_string()));
            known_items.insert(ArrayKey::String("REMOTE_USER".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("REDIRECT_REMOTE_USER".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SCRIPT_FILENAME".to_owned()), (false, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SERVER_ADMIN".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SERVER_PORT".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SERVER_SIGNATURE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("PATH_TRANSLATED".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SCRIPT_NAME".to_owned()), (false, get_non_empty_string()));
            known_items.insert(ArrayKey::String("REQUEST_URI".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("PHP_AUTH_DIGEST".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("PHP_AUTH_USER".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("PHP_AUTH_PW".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("AUTH_TYPE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("PATH_INFO".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("ORIG_PATH_INFO".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("CONTENT_LENGTH".to_owned()), (true, get_string()));
            known_items.insert(ArrayKey::String("CONTENT_TYPE".to_owned()), (true, get_string()));
            // Common, miscellaneous variables
            known_items.insert(ArrayKey::String("FCGI_ROLE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HOME".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_CACHE_CONTROL".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_COOKIE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_PRIORITY".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("PATH".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("REDIRECT_STATUS".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("REQUEST_SCHEME".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("USER".to_owned()), (true, get_non_empty_string()));
            // Common proxy and CDN headers
            known_items
                .insert(ArrayKey::String("HTTP_UPGRADE_INSECURE_REQUESTS".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_X_FORWARDED_PROTO".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_CLIENT_IP".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_X_REAL_IP".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_X_FORWARDED_FOR".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_CF_CONNECTING_IP".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_CF_IPCOUNTRY".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_CF_VISITOR".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_CDN_LOOP".to_owned()), (true, get_non_empty_string()));
            // Common Sec-Fetch headers
            known_items.insert(ArrayKey::String("HTTP_DNT".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_SEC_FETCH_DEST".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_SEC_FETCH_USER".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_SEC_FETCH_MODE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_SEC_FETCH_SITE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_SEC_CH_UA_PLATFORM".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_SEC_CH_UA_MOBILE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_SEC_CH_UA".to_owned()), (true, get_non_empty_string()));

            // Common framework and application variables (e.g., Symfony, PHPUnit, Laravel)
            known_items.insert(ArrayKey::String("APP_DEBUG".to_owned()), (true, get_bool()));
            known_items.insert(ArrayKey::String("APP_ENV".to_owned()), (true, get_string()));
            known_items.insert(ArrayKey::String("APP_NAME".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_URL".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_KEY".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_SECRET".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SECRET".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_LOCALE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_FALLBACK_LOCALE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DATABASE_URL".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("CACHE_DRIVER".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_CONNECTION".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_HOST".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_PORT".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_DATABASE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_USERNAME".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_PASSWORD".to_owned()), (true, get_string()));

            known_items.insert(
                ArrayKey::String("arvc".to_owned()),
                (true, TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::From(1)))])),
            );
            known_items.insert(
                ArrayKey::String("argv".to_owned()),
                (
                    true,
                    TUnion::new(vec![TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                        TAtomic::Scalar(TScalar::string()),
                    ])))))]),
                ),
            );

            TUnion::new(vec![TAtomic::Array(TArray::Keyed(TKeyedArray {
                known_items: Some(known_items),
                parameters: Some((
                    Box::new(TUnion::new(vec![TAtomic::Scalar(TScalar::non_empty_string())])),
                    Box::new(get_string()),
                )),
                non_empty: true,
            }))])
        }
        "$_ENV" => {
            let mut known_items = BTreeMap::new();

            // Standard environment variables
            known_items.insert(ArrayKey::String("PATH".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HOME".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("USER".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SHELL".to_owned()), (true, get_non_empty_string()));

            // Common framework and application variables (e.g., Symfony, PHPUnit, Laravel)
            known_items.insert(ArrayKey::String("APP_DEBUG".to_owned()), (true, get_bool()));
            known_items.insert(ArrayKey::String("APP_ENV".to_owned()), (true, get_string()));
            known_items.insert(ArrayKey::String("APP_NAME".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_URL".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_KEY".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_SECRET".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SECRET".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_LOCALE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_FALLBACK_LOCALE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DATABASE_URL".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("CACHE_DRIVER".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_CONNECTION".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_HOST".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_PORT".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_DATABASE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_USERNAME".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_PASSWORD".to_owned()), (true, get_string()));

            TUnion::new(vec![TAtomic::Array(TArray::Keyed(TKeyedArray {
                known_items: Some(known_items),
                parameters: Some((
                    Box::new(TUnion::new(vec![TAtomic::Scalar(TScalar::non_empty_string())])),
                    Box::new(get_string()),
                )),
                non_empty: true,
            }))])
        }
        "$_FILES" => TUnion::new(vec![TAtomic::Array(TArray::Keyed(TKeyedArray {
            known_items: None,
            parameters: Some((
                Box::new(TUnion::new(vec![TAtomic::Scalar(TScalar::non_empty_string())])),
                Box::new(TUnion::new(vec![TAtomic::Array(TArray::Keyed(TKeyedArray {
                    known_items: Some(BTreeMap::from([
                        (
                            ArrayKey::String("name".to_owned()),
                            (
                                true,
                                TUnion::new(vec![
                                    TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                                        TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    ]))))),
                                ]),
                            ),
                        ),
                        (
                            ArrayKey::String("type".to_owned()),
                            (
                                true,
                                TUnion::new(vec![
                                    TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                                        TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    ]))))),
                                ]),
                            ),
                        ),
                        (
                            ArrayKey::String("tmp_name".to_owned()),
                            (
                                true,
                                TUnion::new(vec![
                                    TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                                        TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    ]))))),
                                ]),
                            ),
                        ),
                        (
                            ArrayKey::String("full_path".to_owned()),
                            (
                                true,
                                TUnion::new(vec![
                                    TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                                        TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    ]))))),
                                ]),
                            ),
                        ),
                        (
                            ArrayKey::String("error".to_owned()),
                            (
                                true,
                                TUnion::new(vec![
                                    TAtomic::Scalar(TScalar::Integer(TInteger::Range(0, 8))),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                                        TAtomic::Scalar(TScalar::Integer(TInteger::Range(0, 8))),
                                    ]))))),
                                ]),
                            ),
                        ),
                        (
                            ArrayKey::String("size".to_owned()),
                            (
                                true,
                                TUnion::new(vec![
                                    TAtomic::Scalar(TScalar::Integer(TInteger::From(0))),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                                        TAtomic::Scalar(TScalar::Integer(TInteger::From(0))),
                                    ]))))),
                                ]),
                            ),
                        ),
                    ])),
                    parameters: None,
                    non_empty: true,
                }))])),
            )),
            non_empty: true,
        }))]),
        "$_SESSION" => TUnion::new(vec![TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
            Box::new(TUnion::new(vec![TAtomic::Scalar(TScalar::non_empty_string())])),
            Box::new(get_mixed()),
        )))]),
        _ => return None,
    })
}

fn find_similar_variable_names(context: &BlockContext<'_>, target: &str) -> Vec<String> {
    let mut suggestions: Vec<(usize, &String)> = Vec::new();

    for local in context.locals.keys() {
        if local.is_empty() {
            continue;
        }

        let distance = strsim::levenshtein(target, local);

        if distance > 0 && distance <= 3 {
            suggestions.push((distance, local));
        }
    }

    suggestions.sort_by_key(|k| k.0);
    suggestions.into_iter().map(|(_, name)| name).cloned().collect()
}

fn generate_confusable_character_note(variable_name: &str) -> Option<String> {
    let mut has_non_std_ascii_alphanumeric = false;
    let mut confusable_examples = Vec::new();

    for c in variable_name.chars().skip(1) {
        if !c.is_ascii_alphanumeric() && c != '_' {
            if c.is_alphabetic() {
                has_non_std_ascii_alphanumeric = true;
                if c == '\u{0430}' {
                    confusable_examples.push("'а' (Cyrillic 'a')");
                } else if c == '\u{03BF}' {
                    confusable_examples.push("'ο' (Greek 'o')");
                }
            } else if c > '\x7F' {
                has_non_std_ascii_alphanumeric = true;
            }
        }
    }

    if has_non_std_ascii_alphanumeric {
        let mut note = format!("Variable name `{variable_name}` contains non-standard ASCII alphanumeric characters.");
        if !confusable_examples.is_empty() {
            note.push_str(&format!(" For example, it might contain {}.", confusable_examples.join(" or ")));
        }

        note.push_str(" Please verify all characters are intended.");

        Some(note)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::issue::TypingIssueKind;
    use crate::test_analysis;

    test_analysis! {
        name = possibly_undefined_variable_from_foreach,
        code = indoc! {r#"
            <?php

            /**
             * @param array<string, string> $arr
             */
            function iter(array $arr)
            {
                $value = 1;
                unset($value);
                foreach ($arr as $key => $value) {
                    $y = 1;
                    echo 'Key: ' . $key . ', Value: ' . $value . "\n";
                    echo 'Y: ' . $y . "\n";
                }

                echo (string) $key;
                echo (string) $value;
                echo (string) $y;
            }
        "#},
        issues = [
            TypingIssueKind::PossiblyUndefinedVariable, // $key
            TypingIssueKind::PossiblyUndefinedVariable, // $value
            TypingIssueKind::PossiblyUndefinedVariable, // $y
        ]
    }

    test_analysis! {
        name = defined_variable_from_foreach,
        code = indoc! {r#"
            <?php

            /**
             * @param non-empty-array<string, string> $arr
             */
            function iter(array $arr)
            {
                $value = 1;
                unset($value);
                foreach ($arr as $key => $value) {
                    $y = 1;
                    echo 'Key: ' . $key . ', Value: ' . $value . "\n";
                    echo 'Y: ' . $y . "\n";
                }

                echo (string) $key;
                echo (string) $value;
                echo (string) $y;
            }
        "#},
        issues = [
            TypingIssueKind::RedundantCast, // $key is known to be a string
            TypingIssueKind::RedundantCast, // $value is known to be a string
        ]
    }
}
