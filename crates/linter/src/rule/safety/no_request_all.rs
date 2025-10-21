use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;
use mago_syntax::utils::reference::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

const REQUEST_CLASS: &str = "Request";
const REQUEST_HELPER: &str = "request";
const REQUEST_FQCN: &str = "Illuminate\\Http\\Request";
const REQUEST_FACADE: &str = "Illuminate\\Support\\Facades\\Request";
const REQUEST_VAR: &str = "$request";
const ALL_METHOD: &str = "all";

#[derive(Debug, Clone)]
pub struct NoRequestAllRule {
    meta: &'static RuleMeta,
    cfg: NoRequestAllConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRequestAllConfig {
    pub level: Level,
}

impl Default for NoRequestAllConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoRequestAllConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRequestAllRule {
    type Config = NoRequestAllConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Request All",
            code: "no-request-all",
            description: indoc! {"
                Detects the use of `$request->all()` or `Request::all()` in Laravel applications.

                Such calls retrieve all input values, including ones you might not expect or intend to handle.
                It is recommended to use `$request->only([...])` to specify the inputs you need explicitly, ensuring better security and validation.
            "},
            good_example: indoc! {r#"
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
            bad_example: indoc! {r#"
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
                        $data = $request->all();

                        // ...
                    }
                }
            "#},
            category: Category::Safety,

            requirements: RuleRequirements::Integration(Integration::Laravel),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Function, NodeKind::Method, NodeKind::Closure];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let block = match node {
            Node::Function(Function { body: block, .. }) => block,
            Node::Method(Method { body: MethodBody::Concrete(block), .. }) => block,
            Node::Closure(Closure { body: block, .. }) => block,
            _ => return,
        };

        let request_all_references = find_method_references_in_block(block, &|reference| {
            let ClassLikeMemberSelector::Identifier(method) = reference.get_selector() else { return false };

            if !method.value.eq_ignore_ascii_case(ALL_METHOD) {
                return false;
            }

            match reference {
                MethodReference::MethodCall(method_call) => match method_call.object {
                    Expression::Variable(Variable::Direct(variable)) => variable.name.eq_ignore_ascii_case(REQUEST_VAR),
                    Expression::Call(Call::Function(call)) if call.argument_list.arguments.is_empty() => {
                        function_call_matches(ctx, call, REQUEST_HELPER)
                    }
                    _ => false,
                },
                MethodReference::StaticMethodCall(static_method_call) => {
                    let Expression::Identifier(identifier) = static_method_call.class else { return false };
                    let fqcn = ctx.lookup_name(identifier);

                    fqcn.eq_ignore_ascii_case(REQUEST_FACADE)
                        || fqcn.eq_ignore_ascii_case(REQUEST_FQCN)
                        || identifier.value().eq_ignore_ascii_case(REQUEST_CLASS)
                }
                _ => false,
            }
        });

        for reference in request_all_references {
            let issue = Issue::new(self.cfg.level(), "Avoid using `$request->all()` or `Request::all()`.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(reference.span()).with_message("`Request::all()` is called here")
                )
                .with_note("Using `$request->all()` retrieves all input values, including ones you might not expect or intend to handle.")
                .with_help("Use `$request->only([...])` to specify the inputs you need explicitly, ensuring better security and validation.");

            ctx.collector.report(issue);
        }
    }
}
