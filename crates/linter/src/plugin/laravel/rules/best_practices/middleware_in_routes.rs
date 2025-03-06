use indoc::indoc;

use mago_ast::*;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::plugin::laravel::rules::utils::is_method_named;
use crate::plugin::laravel::rules::utils::is_this;
use crate::plugin::laravel::rules::utils::is_within_controller;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct MiddlewareInRoutesRule;

impl Rule for MiddlewareInRoutesRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Middleware In Routes", Level::Warning)
            .with_description(indoc! {"
                This rule warns against applying middlewares in controllers.

                Middlewares should be applied in the routes file, not in the controller.
            "})
            .with_example(RuleUsageExample::valid(
                "Applying middleware in the routes file",
                indoc! {r#"
                    <?php

                    // routes/web.php
                    Route::get('/user', 'UserController@index')->middleware('auth');
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Applying middleware in the controller",
                indoc! {r#"
                    <?php

                    namespace App\Http\Controllers;

                    class UserController extends Controller
                    {
                        public function __construct()
                        {
                            $this->middleware('auth');
                        }
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        if !is_within_controller(context) {
            return LintDirective::default();
        }

        let Node::MethodCall(call @ MethodCall { object, method, .. }) = node else {
            return LintDirective::default();
        };

        if !is_this(context, object) || !is_method_named(context, method, "middleware") {
            return LintDirective::default();
        }

        let issue = Issue::new(context.level(), "Avoid applying middlewares in controllers.")
            .with_annotation(Annotation::primary(call.span()).with_message("Middleware applied here."))
            .with_note("Middlewares should be applied in the routes file, not in the controller.")
            .with_help("Move the middleware to the routes file.");

        context.report(issue);

        LintDirective::Prune
    }
}
