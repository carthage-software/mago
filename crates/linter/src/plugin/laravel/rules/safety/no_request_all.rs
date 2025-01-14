use indoc::indoc;

use mago_ast::*;
use mago_ast_utils::reference::*;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::rule::Rule;

const REQUEST_CLASS: &str = "Request";
const REQUEST_HELPER: &str = "request";
const REQUEST_FQCN: &str = "Illuminate\\Http\\Request";
const REQUEST_FACADE: &str = "Illuminate\\Support\\Facades\\Request";
const REQUEST_VAR: &str = "$request";
const ALL_METHOD: &str = "all";

#[derive(Clone, Debug)]
pub struct NoRequestAllRule;

impl Rule for NoRequestAllRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Request All", Level::Warning)
            .with_description(indoc! {"
                Detects the use of `$request->all()` or `Request::all()` in Laravel applications.
                Such calls retrieve all input values, including ones you might not expect or intend to handle.
                It is recommended to use `$request->only([...])` to specify the inputs you need explicitly, ensuring better security and validation.
            "})
            .with_example(RuleUsageExample::valid(
                "Using `$request->only([...])` instead of `$request->all()`",
                indoc! {r#"
                    <?php

                    namespace App\Http\Controllers;

                    use Illuminate\Http\RedirectResponse;
                    use Illuminate\Http\Request;

                    class UserController extends Controller
                    {
                        /**
                         * Store a new user.
                         */
                        public function store(Request $request): RedirectResponse
                        {
                            $data = $request->only(['name', 'email', 'password']);

                            // ...
                        }
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `$request->all()`",
                indoc! {r#"
                    <?php

                    namespace App\Http\Controllers;

                    use Illuminate\Http\RedirectResponse;
                    use Illuminate\Support\Facades\Request;

                    class UserController extends Controller
                    {
                        /**
                         * Store a new user.
                         */
                        public function store(Request $request): RedirectResponse
                        {
                            $data = $request->all();

                            // ...
                        }
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `Request::all()`",
                indoc! {r#"
                    <?php

                    namespace App\Http\Controllers;

                    use Illuminate\Http\RedirectResponse;
                    use Illuminate\Http\Request;

                    class UserController extends Controller
                    {
                        /**
                         * Store a new user.
                         */
                        public function store(): RedirectResponse
                        {
                            $data = Request::all();

                            // ...
                        }
                    }
                "#},
            ))
    }
}

impl<'a> Walker<LintContext<'a>> for NoRequestAllRule {
    fn walk_block(&self, block: &Block, context: &mut LintContext<'a>) {
        let request_all_references = find_method_references_in_block(block, &|reference| {
            let ClassLikeMemberSelector::Identifier(method) = reference.get_selector() else {
                return false;
            };

            if !context.lookup(&method.value).eq_ignore_ascii_case(ALL_METHOD) {
                return false;
            }

            match reference {
                MethodReference::MethodCall(method_call) => match method_call.object.as_ref() {
                    Expression::Variable(Variable::Direct(variable)) => {
                        context.lookup(&variable.name).eq_ignore_ascii_case(REQUEST_VAR)
                    }
                    Expression::Call(Call::Function(FunctionCall { function, arguments }))
                        if arguments.arguments.is_empty() =>
                    {
                        let Expression::Identifier(identifier) = function.as_ref() else {
                            return false;
                        };

                        let name = context.resolve_function_name(identifier);

                        name.eq_ignore_ascii_case(REQUEST_HELPER)
                    }
                    _ => false,
                },
                MethodReference::StaticMethodCall(static_method_call) => {
                    let Expression::Identifier(identifier) = static_method_call.class.as_ref() else {
                        return false;
                    };

                    let fqcn = context.lookup_name(identifier);

                    fqcn.eq_ignore_ascii_case(REQUEST_FACADE)
                        || fqcn.eq_ignore_ascii_case(REQUEST_FQCN)
                        || context.lookup(&identifier.value()).eq_ignore_ascii_case(REQUEST_CLASS)
                }
                _ => {
                    // we do not care about closure creation..
                    false
                }
            }
        });

        for reference in request_all_references {
            let issue = Issue::new(context.level(), "Avoid using `$request->all()` or `Request::all()`.")
                .with_annotation(
                    Annotation::primary(reference.span()).with_message("`Request::all()` is called here")
                )
                .with_note("Using `$request->all()` retrieves all input values, including ones you might not expect or intend to handle.")
                .with_help("Use `$request->only([...])` to specify the inputs you need explicitly, ensuring better security and validation.");

            context.report(issue);
        }
    }
}
