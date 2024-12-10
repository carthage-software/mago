use mago_ast::*;
use mago_span::*;

use crate::context::LintContext;

pub const TESTING_METHODS: [&str; 57] = [
    "anything",
    "arrayHasKey",
    "attribute",
    "attributeEqualTo",
    "callback",
    "classHasAttribute",
    "classHasStaticAttribute",
    "contains",
    "containsEqual",
    "containsIdentical",
    "containsOnly",
    "containsOnlyInstancesOf",
    "countOf",
    "directoryExists",
    "equalTo",
    "equalToCanonicalizing",
    "equalToIgnoringCase",
    "equalToWithDelta",
    "fail",
    "fileExists",
    "getCount",
    "getObjectAttribute",
    "getStaticAttribute",
    "greaterThan",
    "greaterThanOrEqual",
    "identicalTo",
    "isEmpty",
    "isFalse",
    "isFinite",
    "isInfinite",
    "isInstanceOf",
    "isJson",
    "isList",
    "isNan",
    "isNull",
    "isReadable",
    "isTrue",
    "isType",
    "isWritable",
    "lessThan",
    "lessThanOrEqual",
    "logicalAnd",
    "logicalNot",
    "logicalOr",
    "logicalXor",
    "markTestIncomplete",
    "markTestSkipped",
    "matches",
    "matchesRegularExpression",
    "objectEquals",
    "objectHasAttribute",
    "readAttribute",
    "resetCount",
    "stringContains",
    "stringEndsWith",
    "stringEqualsStringIgnoringLineEndings",
    "stringStartsWith",
];

pub const ASSERTION_METHODS: [&str; 173] = [
    "assertArrayHasKey",
    "assertArrayIsEqualToArrayIgnoringListOfKeys",
    "assertArrayIsEqualToArrayOnlyConsideringListOfKeys",
    "assertArrayIsIdenticalToArrayIgnoringListOfKeys",
    "assertArrayIsIdenticalToArrayOnlyConsideringListOfKeys",
    "assertArrayNotHasKey",
    "assertArraySubset",
    "assertAttributeContains",
    "assertAttributeContainsOnly",
    "assertAttributeCount",
    "assertAttributeEmpty",
    "assertAttributeEquals",
    "assertAttributeGreaterThan",
    "assertAttributeGreaterThanOrEqual",
    "assertAttributeInstanceOf",
    "assertAttributeInternalType",
    "assertAttributeLessThan",
    "assertAttributeLessThanOrEqual",
    "assertAttributeNotContains",
    "assertAttributeNotContainsOnly",
    "assertAttributeNotCount",
    "assertAttributeNotEmpty",
    "assertAttributeNotEquals",
    "assertAttributeNotInstanceOf",
    "assertAttributeNotInternalType",
    "assertAttributeNotSame",
    "assertAttributeSame",
    "assertClassHasAttribute",
    "assertClassHasStaticAttribute",
    "assertClassNotHasAttribute",
    "assertClassNotHasStaticAttribute",
    "assertContains",
    "assertContainsEquals",
    "assertContainsOnly",
    "assertContainsOnlyInstancesOf",
    "assertCount",
    "assertDirectoryDoesNotExist",
    "assertDirectoryExists",
    "assertDirectoryIsNotReadable",
    "assertDirectoryIsNotWritable",
    "assertDirectoryIsReadable",
    "assertDirectoryIsWritable",
    "assertDirectoryNotExists",
    "assertDirectoryNotIsReadable",
    "assertDirectoryNotIsWritable",
    "assertDoesNotMatchRegularExpression",
    "assertEmpty",
    "assertEquals",
    "assertEqualsCanonicalizing",
    "assertEqualsIgnoringCase",
    "assertEqualsWithDelta",
    "assertEqualXMLStructure",
    "assertFalse",
    "assertFileDoesNotExist",
    "assertFileEquals",
    "assertFileEqualsCanonicalizing",
    "assertFileEqualsIgnoringCase",
    "assertFileExists",
    "assertFileIsNotReadable",
    "assertFileIsNotWritable",
    "assertFileIsReadable",
    "assertFileIsWritable",
    "assertFileMatchesFormat",
    "assertFileMatchesFormatFile",
    "assertFileNotEquals",
    "assertFileNotEqualsCanonicalizing",
    "assertFileNotEqualsIgnoringCase",
    "assertFileNotExists",
    "assertFileNotIsReadable",
    "assertFileNotIsWritable",
    "assertFinite",
    "assertGreaterThan",
    "assertGreaterThanOrEqual",
    "assertInfinite",
    "assertInstanceOf",
    "assertInternalType",
    "assertIsArray",
    "assertIsBool",
    "assertIsCallable",
    "assertIsClosedResource",
    "assertIsFloat",
    "assertIsInt",
    "assertIsIterable",
    "assertIsList",
    "assertIsNotArray",
    "assertIsNotBool",
    "assertIsNotCallable",
    "assertIsNotClosedResource",
    "assertIsNotFloat",
    "assertIsNotInt",
    "assertIsNotIterable",
    "assertIsNotNumeric",
    "assertIsNotObject",
    "assertIsNotReadable",
    "assertIsNotResource",
    "assertIsNotScalar",
    "assertIsNotString",
    "assertIsNotWritable",
    "assertIsNumeric",
    "assertIsObject",
    "assertIsReadable",
    "assertIsResource",
    "assertIsScalar",
    "assertIsString",
    "assertIsWritable",
    "assertJson",
    "assertJsonFileEqualsJsonFile",
    "assertJsonFileNotEqualsJsonFile",
    "assertJsonStringEqualsJsonFile",
    "assertJsonStringEqualsJsonString",
    "assertJsonStringNotEqualsJsonFile",
    "assertJsonStringNotEqualsJsonString",
    "assertLessThan",
    "assertLessThanOrEqual",
    "assertMatchesRegularExpression",
    "assertNan",
    "assertNotContains",
    "assertNotContainsEquals",
    "assertNotContainsOnly",
    "assertNotCount",
    "assertNotEmpty",
    "assertNotEquals",
    "assertNotEqualsCanonicalizing",
    "assertNotEqualsIgnoringCase",
    "assertNotEqualsWithDelta",
    "assertNotFalse",
    "assertNotInstanceOf",
    "assertNotInternalType",
    "assertNotIsReadable",
    "assertNotIsWritable",
    "assertNotNull",
    "assertNotRegExp",
    "assertNotSame",
    "assertNotSameSize",
    "assertNotTrue",
    "assertNull",
    "assertObjectEquals",
    "assertObjectHasAttribute",
    "assertObjectHasProperty",
    "assertObjectNotEquals",
    "assertObjectNotHasAttribute",
    "assertObjectNotHasProperty",
    "assertRegExp",
    "assertSame",
    "assertSameSize",
    "assertStringContainsString",
    "assertStringContainsStringIgnoringCase",
    "assertStringContainsStringIgnoringLineEndings",
    "assertStringEndsNotWith",
    "assertStringEndsWith",
    "assertStringEqualsFile",
    "assertStringEqualsFileCanonicalizing",
    "assertStringEqualsFileIgnoringCase",
    "assertStringEqualsStringIgnoringLineEndings",
    "assertStringMatchesFormat",
    "assertStringMatchesFormatFile",
    "assertStringNotContainsString",
    "assertStringNotContainsStringIgnoringCase",
    "assertStringNotEqualsFile",
    "assertStringNotEqualsFileCanonicalizing",
    "assertStringNotEqualsFileIgnoringCase",
    "assertStringNotMatchesFormat",
    "assertStringNotMatchesFormatFile",
    "assertStringStartsNotWith",
    "assertStringStartsWith",
    "assertThat",
    "assertTrue",
    "assertXmlFileEqualsXmlFile",
    "assertXmlFileNotEqualsXmlFile",
    "assertXmlStringEqualsXmlFile",
    "assertXmlStringEqualsXmlString",
    "assertXmlStringNotEqualsXmlFile",
    "assertXmlStringNotEqualsXmlString",
];

pub(super) enum MethodReference<'a> {
    MethodCall(&'a MethodCall),
    StaticMethodCall(&'a StaticMethodCall),
    MethodClosureCreation(&'a MethodClosureCreation),
    StaticMethodClosureCreation(&'a StaticMethodClosureCreation),
}

impl MethodReference<'_> {
    pub fn get_class_or_object(&self) -> &Expression {
        match self {
            MethodReference::MethodCall(call) => &call.object,
            MethodReference::StaticMethodCall(call) => &call.class,
            MethodReference::MethodClosureCreation(closure) => &closure.object,
            MethodReference::StaticMethodClosureCreation(closure) => &closure.class,
        }
    }

    pub fn get_selector(&self) -> &ClassLikeMemberSelector {
        match self {
            MethodReference::MethodCall(call) => &call.method,
            MethodReference::StaticMethodCall(call) => &call.method,
            MethodReference::MethodClosureCreation(closure) => &closure.method,
            MethodReference::StaticMethodClosureCreation(closure) => &closure.method,
        }
    }
}

impl HasSpan for MethodReference<'_> {
    fn span(&self) -> Span {
        match self {
            MethodReference::MethodCall(call) => call.span(),
            MethodReference::StaticMethodCall(call) => call.span(),
            MethodReference::MethodClosureCreation(closure) => closure.span(),
            MethodReference::StaticMethodClosureCreation(closure) => closure.span(),
        }
    }
}

pub fn find_assertions_methods<'a>(method: &'a Method, context: &LintContext<'_>) -> Vec<MethodReference<'a>> {
    find_method_references_matching(method, context, |reference, context| {
        let class_or_object = reference.get_class_or_object();

        if let Expression::Variable(Variable::Direct(variable)) = class_or_object {
            if context.lookup(&variable.name) != "$this" {
                return false;
            }
        } else if !matches!(class_or_object, Expression::Static(_) | Expression::Self_(_)) {
            return false;
        }

        let ClassLikeMemberSelector::Identifier(identifier) = reference.get_selector() else {
            return false;
        };

        let name = context.lookup(&identifier.value);

        ASSERTION_METHODS.contains(&name)
    })
}

pub fn find_testing_and_assertions_methods<'a>(
    method: &'a Method,
    context: &LintContext<'_>,
) -> Vec<MethodReference<'a>> {
    find_method_references_matching(method, context, |reference, context| {
        let class_or_object = reference.get_class_or_object();

        if let Expression::Variable(Variable::Direct(variable)) = class_or_object {
            if context.lookup(&variable.name) != "$this" {
                return false;
            }
        } else if !matches!(class_or_object, Expression::Static(_) | Expression::Self_(_)) {
            return false;
        }

        let ClassLikeMemberSelector::Identifier(identifier) = reference.get_selector() else {
            return false;
        };

        let name = context.lookup(&identifier.value);

        ASSERTION_METHODS.contains(&name) || TESTING_METHODS.contains(&name)
    })
}

fn find_method_references_matching<'a, F>(
    method: &'a Method,
    context: &LintContext<'_>,
    predicate: F,
) -> Vec<MethodReference<'a>>
where
    F: Fn(&MethodReference<'a>, &LintContext<'_>) -> bool,
{
    let MethodBody::Concrete(block) = &method.body else {
        return vec![];
    };

    let mut method_references = vec![];
    for statement in block.statements.iter() {
        method_references.extend(find_references_in_statement(statement, context, &predicate));
    }

    method_references
}

fn find_references_in_statement<'a, F>(
    statement: &'a Statement,
    context: &LintContext<'_>,
    predicate: &F,
) -> Vec<MethodReference<'a>>
where
    F: Fn(&MethodReference<'a>, &LintContext<'_>) -> bool,
{
    match statement {
        Statement::Block(block) => {
            let mut references = vec![];
            for statement in block.statements.iter() {
                references.extend(find_references_in_statement(statement, context, predicate));
            }

            references
        }
        Statement::Try(try_catch) => {
            let mut references = vec![];
            for statement in try_catch.block.statements.iter() {
                references.extend(find_references_in_statement(statement, context, predicate));
            }

            for catch in try_catch.catch_clauses.iter() {
                for statement in catch.block.statements.iter() {
                    references.extend(find_references_in_statement(statement, context, predicate));
                }
            }

            if let Some(finally) = &try_catch.finally_clause {
                for statement in finally.block.statements.iter() {
                    references.extend(find_references_in_statement(statement, context, predicate));
                }
            }

            references
        }
        Statement::Foreach(foreach) => {
            let mut references = vec![];

            references.extend(find_references_in_expression(&foreach.expression, context, predicate));

            match &foreach.target {
                ForeachTarget::Value(foreach_value_target) => {
                    references.extend(find_references_in_expression(&foreach_value_target.value, context, predicate));
                }
                ForeachTarget::KeyValue(foreach_key_value_target) => {
                    references.extend(find_references_in_expression(&foreach_key_value_target.key, context, predicate));
                    references.extend(find_references_in_expression(
                        &foreach_key_value_target.value,
                        context,
                        predicate,
                    ));
                }
            }

            match &foreach.body {
                ForeachBody::Statement(statement) => {
                    references.extend(find_references_in_statement(statement, context, predicate));
                }
                ForeachBody::ColonDelimited(foreach_colon_delimited_body) => {
                    for statement in foreach_colon_delimited_body.statements.iter() {
                        references.extend(find_references_in_statement(statement, context, predicate));
                    }
                }
            }

            references
        }
        Statement::For(for_loop) => {
            let mut references = vec![];

            for init in for_loop.initializations.iter() {
                references.extend(find_references_in_expression(init, context, predicate));
            }

            for condition in for_loop.conditions.iter() {
                references.extend(find_references_in_expression(condition, context, predicate));
            }

            for increment in for_loop.increments.iter() {
                references.extend(find_references_in_expression(increment, context, predicate));
            }

            match &for_loop.body {
                ForBody::Statement(statement) => {
                    references.extend(find_references_in_statement(statement, context, predicate));
                }
                ForBody::ColonDelimited(for_colon_delimited_body) => {
                    for statement in for_colon_delimited_body.statements.iter() {
                        references.extend(find_references_in_statement(statement, context, predicate));
                    }
                }
            }

            references
        }
        Statement::While(while_loop) => {
            let mut references = vec![];

            references.extend(find_references_in_expression(&while_loop.condition, context, predicate));

            match &while_loop.body {
                WhileBody::Statement(statement) => {
                    references.extend(find_references_in_statement(statement, context, predicate));
                }
                WhileBody::ColonDelimited(while_colon_delimited_body) => {
                    for statement in while_colon_delimited_body.statements.iter() {
                        references.extend(find_references_in_statement(statement, context, predicate));
                    }
                }
            }

            references
        }
        Statement::DoWhile(do_while) => {
            let mut references = vec![];

            references.extend(find_references_in_expression(&do_while.condition, context, predicate));
            references.extend(find_references_in_statement(&do_while.statement, context, predicate));

            references
        }
        Statement::Switch(switch) => {
            let mut references = find_references_in_expression(&switch.expression, context, predicate);

            for case in switch.body.cases() {
                match case {
                    SwitchCase::Expression(expression_case) => {
                        references.extend(find_references_in_expression(
                            &expression_case.expression,
                            context,
                            predicate,
                        ));

                        for statement in expression_case.statements.iter() {
                            references.extend(find_references_in_statement(statement, context, predicate));
                        }
                    }
                    SwitchCase::Default(default_case) => {
                        for statement in default_case.statements.iter() {
                            references.extend(find_references_in_statement(statement, context, predicate));
                        }
                    }
                }
            }

            references
        }
        Statement::If(if_stmt) => {
            let mut references = vec![];

            references.extend(find_references_in_expression(&if_stmt.condition, context, predicate));
            match &if_stmt.body {
                IfBody::Statement(if_stmt_body) => {
                    references.extend(find_references_in_statement(&if_stmt_body.statement, context, predicate));
                    for else_if_clause in if_stmt_body.else_if_clauses.iter() {
                        references.extend(find_references_in_expression(&else_if_clause.condition, context, predicate));
                        references.extend(find_references_in_statement(&else_if_clause.statement, context, predicate));
                    }

                    if let Some(else_clause) = &if_stmt_body.else_clause {
                        references.extend(find_references_in_statement(&else_clause.statement, context, predicate));
                    }
                }
                IfBody::ColonDelimited(if_colon_delimited_body) => {
                    for statement in if_colon_delimited_body.statements.iter() {
                        references.extend(find_references_in_statement(statement, context, predicate));
                    }

                    for else_if_clause in if_colon_delimited_body.else_if_clauses.iter() {
                        references.extend(find_references_in_expression(&else_if_clause.condition, context, predicate));
                        for statement in else_if_clause.statements.iter() {
                            references.extend(find_references_in_statement(statement, context, predicate));
                        }
                    }

                    if let Some(else_clause) = &if_colon_delimited_body.else_clause {
                        for statement in else_clause.statements.iter() {
                            references.extend(find_references_in_statement(statement, context, predicate));
                        }
                    }
                }
            }

            references
        }
        Statement::Return(r#return) => {
            if let Some(expression) = &r#return.value {
                find_references_in_expression(expression, context, predicate)
            } else {
                vec![]
            }
        }
        Statement::Expression(expression_statement) => {
            find_references_in_expression(&expression_statement.expression, context, predicate)
        }
        Statement::Echo(echo) => {
            let mut references = vec![];
            for expression in echo.values.iter() {
                references.extend(find_references_in_expression(expression, context, predicate));
            }

            references
        }
        _ => {
            vec![]
        }
    }
}

fn find_references_in_expression<'a, F>(
    expression: &'a Expression,
    context: &LintContext<'_>,
    predicate: &F,
) -> Vec<MethodReference<'a>>
where
    F: Fn(&MethodReference<'a>, &LintContext<'_>) -> bool,
{
    match expression {
        Expression::Binary(binary) => {
            let mut references = vec![];
            references.extend(find_references_in_expression(binary.lhs.as_ref(), context, predicate));
            references.extend(find_references_in_expression(binary.rhs.as_ref(), context, predicate));

            references
        }
        Expression::UnaryPrefix(unary_prefix) => {
            find_references_in_expression(unary_prefix.operand.as_ref(), context, predicate)
        }
        Expression::UnaryPostfix(unary_postfix) => {
            find_references_in_expression(unary_postfix.operand.as_ref(), context, predicate)
        }
        Expression::Parenthesized(parenthesized) => {
            find_references_in_expression(parenthesized.expression.as_ref(), context, predicate)
        }
        Expression::AssignmentOperation(assignment) => {
            let mut references = vec![];
            references.extend(find_references_in_expression(assignment.lhs.as_ref(), context, predicate));
            references.extend(find_references_in_expression(assignment.rhs.as_ref(), context, predicate));

            references
        }
        Expression::Conditional(conditional) => {
            let mut references = vec![];
            references.extend(find_references_in_expression(conditional.condition.as_ref(), context, predicate));
            if let Some(then) = &conditional.then {
                references.extend(find_references_in_expression(then, context, predicate));
            }
            references.extend(find_references_in_expression(conditional.r#else.as_ref(), context, predicate));

            references
        }
        Expression::Array(Array { elements, .. })
        | Expression::LegacyArray(LegacyArray { elements, .. })
        | Expression::List(List { elements, .. }) => {
            let mut references = vec![];
            for element in elements.iter() {
                match element {
                    ArrayElement::KeyValue(kv) => {
                        references.extend(find_references_in_expression(kv.key.as_ref(), context, predicate));
                        references.extend(find_references_in_expression(kv.value.as_ref(), context, predicate));
                    }
                    ArrayElement::Value(v) => {
                        references.extend(find_references_in_expression(v.value.as_ref(), context, predicate));
                    }
                    ArrayElement::Variadic(v) => {
                        references.extend(find_references_in_expression(v.value.as_ref(), context, predicate));
                    }
                    ArrayElement::Missing(_) => {}
                }
            }

            references
        }
        Expression::ArrayAccess(array_access) => {
            let mut references = vec![];
            references.extend(find_references_in_expression(array_access.array.as_ref(), context, predicate));
            references.extend(find_references_in_expression(array_access.index.as_ref(), context, predicate));

            references
        }
        Expression::ArrayAppend(array_append) => {
            find_references_in_expression(array_append.array.as_ref(), context, predicate)
        }
        Expression::AnonymousClass(anonymous_class) => {
            if let Some(arguments) = &anonymous_class.arguments {
                find_references_in_argument_list(arguments, context, predicate)
            } else {
                vec![]
            }
        }
        Expression::Match(r#match) => {
            let mut references = vec![];
            references.extend(find_references_in_expression(&r#match.expression, context, predicate));

            for arm in r#match.arms.iter() {
                match arm {
                    MatchArm::Expression(match_expression_arm) => {
                        for condition in match_expression_arm.conditions.iter() {
                            references.extend(find_references_in_expression(condition, context, predicate));
                        }

                        references.extend(find_references_in_expression(
                            &match_expression_arm.expression,
                            context,
                            predicate,
                        ));
                    }
                    MatchArm::Default(match_default_arm) => {
                        references.extend(find_references_in_expression(
                            &match_default_arm.expression,
                            context,
                            predicate,
                        ));
                    }
                }
            }

            references
        }
        Expression::Yield(r#yield) => match r#yield.as_ref() {
            Yield::Value(yield_value) => match &yield_value.value {
                Some(value) => find_references_in_expression(value, context, predicate),
                None => vec![],
            },
            Yield::Pair(yield_pair) => {
                let mut references = vec![];
                references.extend(find_references_in_expression(&yield_pair.key, context, predicate));
                references.extend(find_references_in_expression(&yield_pair.value, context, predicate));

                references
            }
            Yield::From(yield_from) => find_references_in_expression(&yield_from.iterator, context, predicate),
        },
        Expression::Throw(throw) => find_references_in_expression(&throw.exception, context, predicate),
        Expression::Clone(clone) => find_references_in_expression(&clone.object, context, predicate),
        Expression::Call(call) => match call {
            Call::Function(function_call) => {
                let mut references = vec![];
                references.extend(find_references_in_expression(&function_call.function, context, predicate));
                references.extend(find_references_in_argument_list(&function_call.arguments, context, predicate));

                references
            }
            Call::Method(method_call) => {
                let reference = MethodReference::MethodCall(method_call);
                let mut references = if predicate(&reference, context) { vec![reference] } else { vec![] };
                references.extend(find_references_in_expression(&method_call.object, context, predicate));
                references.extend(find_references_in_argument_list(&method_call.arguments, context, predicate));

                references
            }
            Call::NullSafeMethod(null_safe_method_call) => {
                let mut references = vec![];
                references.extend(find_references_in_expression(&null_safe_method_call.object, context, predicate));
                references.extend(find_references_in_argument_list(
                    &null_safe_method_call.arguments,
                    context,
                    predicate,
                ));

                references
            }
            Call::StaticMethod(static_method_call) => {
                let mut references = vec![];
                references.extend(find_references_in_expression(&static_method_call.class, context, predicate));
                references.extend(find_references_in_argument_list(&static_method_call.arguments, context, predicate));
                let reference = MethodReference::StaticMethodCall(static_method_call);
                if predicate(&reference, context) {
                    references.push(reference);
                }

                references
            }
        },
        Expression::ClosureCreation(closure_creation) => match closure_creation.as_ref() {
            ClosureCreation::Method(method_closure_creation) => {
                let reference = MethodReference::MethodClosureCreation(method_closure_creation);
                let mut references = if predicate(&reference, context) { vec![reference] } else { vec![] };

                references.extend(find_references_in_expression(&method_closure_creation.object, context, predicate));
                references
            }
            ClosureCreation::StaticMethod(static_method_closure_creation) => {
                let reference = MethodReference::StaticMethodClosureCreation(static_method_closure_creation);
                let mut references = if predicate(&reference, context) { vec![reference] } else { vec![] };

                references.extend(find_references_in_expression(
                    &static_method_closure_creation.class,
                    context,
                    predicate,
                ));
                references
            }
            ClosureCreation::Function(_) => vec![],
        },
        Expression::Instantiation(instantiation) => {
            if let Some(arguments) = &instantiation.arguments {
                find_references_in_argument_list(arguments, context, predicate)
            } else {
                vec![]
            }
        }
        _ => {
            vec![]
        }
    }
}

pub fn find_references_in_argument_list<'a, F>(
    argument_list: &'a ArgumentList,
    context: &LintContext<'_>,
    predicate: &F,
) -> Vec<MethodReference<'a>>
where
    F: Fn(&MethodReference<'a>, &LintContext<'_>) -> bool,
{
    let mut references = vec![];
    for argument in argument_list.arguments.iter() {
        match argument {
            Argument::Positional(positional_argument) => {
                references.extend(find_references_in_expression(&positional_argument.value, context, predicate));
            }
            Argument::Named(named_argument) => {
                references.extend(find_references_in_expression(&named_argument.value, context, predicate));
            }
        }
    }

    references
}
