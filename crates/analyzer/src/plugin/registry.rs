//! Plugin registry for managing and dispatching to providers and hooks.

use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::AtomSet;
use mago_atom::ascii_lowercase_atom;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::metadata::property::PropertyMetadata;
use mago_codex::ttype::union::TUnion;
use mago_database::file::File;
use mago_syntax::ast::*;

use crate::artifacts::AnalysisArtifacts;
use crate::context::block::BlockContext;
use crate::invocation::Invocation;
use crate::plugin::context::HookContext;
use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::context::ReportedIssue;
use crate::plugin::error::PluginResult;
use crate::plugin::hook::*;
use crate::plugin::provider::assertion::FunctionAssertionProvider;
use crate::plugin::provider::assertion::InvocationAssertions;
use crate::plugin::provider::assertion::MethodAssertionProvider;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;
use crate::plugin::provider::method::MethodReturnTypeProvider;
use crate::plugin::provider::method::MethodTarget;
use crate::plugin::provider::property::PropertyInitializationProvider;
use crate::plugin::provider::throw::ExpressionThrowTypeProvider;
use crate::plugin::provider::throw::FunctionThrowTypeProvider;
use crate::plugin::provider::throw::MethodThrowTypeProvider;

use mago_reporting::IssueCollection;

pub struct ProviderResult {
    pub return_type: Option<TUnion>,
    pub issues: Vec<ReportedIssue>,
}

#[derive(Default)]
pub struct PluginRegistry {
    function_exact: AtomMap<Vec<usize>>,
    function_prefix: Vec<(Atom, usize)>,
    function_namespace: Vec<(Atom, usize)>,
    function_providers: Vec<Box<dyn FunctionReturnTypeProvider>>,
    method_exact: AtomMap<Vec<usize>>,
    method_wildcard: Vec<(Vec<MethodTarget>, usize)>,
    method_providers: Vec<Box<dyn MethodReturnTypeProvider>>,
    program_hooks: Vec<Box<dyn ProgramHook>>,
    statement_hooks: Vec<Box<dyn StatementHook>>,
    expression_hooks: Vec<Box<dyn ExpressionHook>>,
    function_call_hooks: Vec<Box<dyn FunctionCallHook>>,
    method_call_hooks: Vec<Box<dyn MethodCallHook>>,
    static_method_call_hooks: Vec<Box<dyn StaticMethodCallHook>>,
    nullsafe_method_call_hooks: Vec<Box<dyn NullSafeMethodCallHook>>,
    class_hooks: Vec<Box<dyn ClassDeclarationHook>>,
    interface_hooks: Vec<Box<dyn InterfaceDeclarationHook>>,
    trait_hooks: Vec<Box<dyn TraitDeclarationHook>>,
    enum_hooks: Vec<Box<dyn EnumDeclarationHook>>,
    function_decl_hooks: Vec<Box<dyn FunctionDeclarationHook>>,
    property_initialization_providers: Vec<Box<dyn PropertyInitializationProvider>>,
    issue_filter_hooks: Vec<Box<dyn IssueFilterHook>>,
    function_assertion_exact: AtomMap<Vec<usize>>,
    function_assertion_prefix: Vec<(Atom, usize)>,
    function_assertion_namespace: Vec<(Atom, usize)>,
    function_assertion_providers: Vec<Box<dyn FunctionAssertionProvider>>,
    method_assertion_exact: AtomMap<Vec<usize>>,
    method_assertion_wildcard: Vec<(Vec<MethodTarget>, usize)>,
    method_assertion_providers: Vec<Box<dyn MethodAssertionProvider>>,
    expression_throw_providers: Vec<Box<dyn ExpressionThrowTypeProvider>>,
    function_throw_exact: AtomMap<Vec<usize>>,
    function_throw_prefix: Vec<(Atom, usize)>,
    function_throw_namespace: Vec<(Atom, usize)>,
    function_throw_providers: Vec<Box<dyn FunctionThrowTypeProvider>>,
    method_throw_exact: AtomMap<Vec<usize>>,
    method_throw_wildcard: Vec<(Vec<MethodTarget>, usize)>,
    method_throw_providers: Vec<Box<dyn MethodThrowTypeProvider>>,
}

impl std::fmt::Debug for PluginRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginRegistry")
            .field("function_providers", &self.function_providers.len())
            .field("method_providers", &self.method_providers.len())
            .field("program_hooks", &self.program_hooks.len())
            .field("statement_hooks", &self.statement_hooks.len())
            .field("expression_hooks", &self.expression_hooks.len())
            .field("function_call_hooks", &self.function_call_hooks.len())
            .field("method_call_hooks", &self.method_call_hooks.len())
            .field("static_method_call_hooks", &self.static_method_call_hooks.len())
            .field("nullsafe_method_call_hooks", &self.nullsafe_method_call_hooks.len())
            .field("class_hooks", &self.class_hooks.len())
            .field("interface_hooks", &self.interface_hooks.len())
            .field("trait_hooks", &self.trait_hooks.len())
            .field("enum_hooks", &self.enum_hooks.len())
            .field("function_decl_hooks", &self.function_decl_hooks.len())
            .field("property_initialization_providers", &self.property_initialization_providers.len())
            .field("issue_filter_hooks", &self.issue_filter_hooks.len())
            .field("function_assertion_providers", &self.function_assertion_providers.len())
            .field("method_assertion_providers", &self.method_assertion_providers.len())
            .field("expression_throw_providers", &self.expression_throw_providers.len())
            .field("function_throw_providers", &self.function_throw_providers.len())
            .field("method_throw_providers", &self.method_throw_providers.len())
            .finish()
    }
}

impl PluginRegistry {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_library_providers() -> Self {
        crate::plugin::create_registry()
    }

    pub fn register_function_provider<P: FunctionReturnTypeProvider + 'static>(&mut self, provider: P) {
        let index = self.function_providers.len();

        match P::targets() {
            FunctionTarget::Exact(name) => {
                self.function_exact.entry(ascii_lowercase_atom(name)).or_default().push(index);
            }
            FunctionTarget::ExactMultiple(names) => {
                for name in names {
                    self.function_exact.entry(ascii_lowercase_atom(name)).or_default().push(index);
                }
            }
            FunctionTarget::Prefix(prefix) => {
                self.function_prefix.push((ascii_lowercase_atom(prefix), index));
            }
            FunctionTarget::Namespace(ns) => {
                let ns_lower = ns.to_lowercase();
                let ns_pattern = if ns_lower.ends_with('\\') { ns_lower } else { format!("{}\\", ns_lower) };
                self.function_namespace.push((ascii_lowercase_atom(&ns_pattern), index));
            }
        }

        self.function_providers.push(Box::new(provider));
    }

    pub fn register_method_provider<P: MethodReturnTypeProvider + 'static>(&mut self, provider: P) {
        let index = self.method_providers.len();
        let targets = P::targets();

        let mut has_wildcards = false;
        let mut wildcard_targets = Vec::new();

        for target in targets {
            if let Some(key) = target.index_key() {
                self.method_exact.entry(key).or_default().push(index);
            } else {
                has_wildcards = true;
                wildcard_targets.push(*target);
            }
        }

        if has_wildcards {
            self.method_wildcard.push((wildcard_targets, index));
        }

        self.method_providers.push(Box::new(provider));
    }

    pub fn register_program_hook<H: ProgramHook + 'static>(&mut self, hook: H) {
        self.program_hooks.push(Box::new(hook));
    }

    pub fn register_statement_hook<H: StatementHook + 'static>(&mut self, hook: H) {
        self.statement_hooks.push(Box::new(hook));
    }

    pub fn register_expression_hook<H: ExpressionHook + 'static>(&mut self, hook: H) {
        self.expression_hooks.push(Box::new(hook));
    }

    pub fn register_function_call_hook<H: FunctionCallHook + 'static>(&mut self, hook: H) {
        self.function_call_hooks.push(Box::new(hook));
    }

    pub fn register_method_call_hook<H: MethodCallHook + 'static>(&mut self, hook: H) {
        self.method_call_hooks.push(Box::new(hook));
    }

    pub fn register_static_method_call_hook<H: StaticMethodCallHook + 'static>(&mut self, hook: H) {
        self.static_method_call_hooks.push(Box::new(hook));
    }

    pub fn register_nullsafe_method_call_hook<H: NullSafeMethodCallHook + 'static>(&mut self, hook: H) {
        self.nullsafe_method_call_hooks.push(Box::new(hook));
    }

    pub fn register_class_hook<H: ClassDeclarationHook + 'static>(&mut self, hook: H) {
        self.class_hooks.push(Box::new(hook));
    }

    pub fn register_interface_hook<H: InterfaceDeclarationHook + 'static>(&mut self, hook: H) {
        self.interface_hooks.push(Box::new(hook));
    }

    pub fn register_trait_hook<H: TraitDeclarationHook + 'static>(&mut self, hook: H) {
        self.trait_hooks.push(Box::new(hook));
    }

    pub fn register_enum_hook<H: EnumDeclarationHook + 'static>(&mut self, hook: H) {
        self.enum_hooks.push(Box::new(hook));
    }

    pub fn register_function_decl_hook<H: FunctionDeclarationHook + 'static>(&mut self, hook: H) {
        self.function_decl_hooks.push(Box::new(hook));
    }

    pub fn register_property_initialization_provider<P: PropertyInitializationProvider + 'static>(
        &mut self,
        provider: P,
    ) {
        self.property_initialization_providers.push(Box::new(provider));
    }

    pub fn register_issue_filter_hook<H: IssueFilterHook + 'static>(&mut self, hook: H) {
        self.issue_filter_hooks.push(Box::new(hook));
    }

    pub fn register_function_assertion_provider<P: FunctionAssertionProvider + 'static>(&mut self, provider: P) {
        let index = self.function_assertion_providers.len();

        match P::targets() {
            FunctionTarget::Exact(name) => {
                self.function_assertion_exact.entry(ascii_lowercase_atom(name)).or_default().push(index);
            }
            FunctionTarget::ExactMultiple(names) => {
                for name in names {
                    self.function_assertion_exact.entry(ascii_lowercase_atom(name)).or_default().push(index);
                }
            }
            FunctionTarget::Prefix(prefix) => {
                self.function_assertion_prefix.push((ascii_lowercase_atom(prefix), index));
            }
            FunctionTarget::Namespace(ns) => {
                let ns_lower = ns.to_lowercase();
                let ns_pattern = if ns_lower.ends_with('\\') { ns_lower } else { format!("{}\\", ns_lower) };
                self.function_assertion_namespace.push((ascii_lowercase_atom(&ns_pattern), index));
            }
        }

        self.function_assertion_providers.push(Box::new(provider));
    }

    pub fn register_method_assertion_provider<P: MethodAssertionProvider + 'static>(&mut self, provider: P) {
        let index = self.method_assertion_providers.len();
        let targets = P::targets();

        let mut has_wildcards = false;
        let mut wildcard_targets = Vec::new();

        for target in targets {
            if let Some(key) = target.index_key() {
                self.method_assertion_exact.entry(key).or_default().push(index);
            } else {
                has_wildcards = true;
                wildcard_targets.push(*target);
            }
        }

        if has_wildcards {
            self.method_assertion_wildcard.push((wildcard_targets, index));
        }

        self.method_assertion_providers.push(Box::new(provider));
    }

    pub fn register_expression_throw_provider<P: ExpressionThrowTypeProvider + 'static>(&mut self, provider: P) {
        self.expression_throw_providers.push(Box::new(provider));
    }

    pub fn register_function_throw_provider<P: FunctionThrowTypeProvider + 'static>(&mut self, provider: P) {
        let index = self.function_throw_providers.len();

        match P::targets() {
            FunctionTarget::Exact(name) => {
                self.function_throw_exact.entry(ascii_lowercase_atom(name)).or_default().push(index);
            }
            FunctionTarget::ExactMultiple(names) => {
                for name in names {
                    self.function_throw_exact.entry(ascii_lowercase_atom(name)).or_default().push(index);
                }
            }
            FunctionTarget::Prefix(prefix) => {
                self.function_throw_prefix.push((ascii_lowercase_atom(prefix), index));
            }
            FunctionTarget::Namespace(ns) => {
                let ns_lower = ns.to_lowercase();
                let ns_pattern = if ns_lower.ends_with('\\') { ns_lower } else { format!("{}\\", ns_lower) };
                self.function_throw_namespace.push((ascii_lowercase_atom(&ns_pattern), index));
            }
        }

        self.function_throw_providers.push(Box::new(provider));
    }

    pub fn register_method_throw_provider<P: MethodThrowTypeProvider + 'static>(&mut self, provider: P) {
        let index = self.method_throw_providers.len();
        let targets = P::targets();

        let mut has_wildcards = false;
        let mut wildcard_targets = Vec::new();

        for target in targets {
            if let Some(key) = target.index_key() {
                self.method_throw_exact.entry(key).or_default().push(index);
            } else {
                has_wildcards = true;
                wildcard_targets.push(*target);
            }
        }

        if has_wildcards {
            self.method_throw_wildcard.push((wildcard_targets, index));
        }

        self.method_throw_providers.push(Box::new(provider));
    }

    #[inline]
    pub fn has_program_hooks(&self) -> bool {
        !self.program_hooks.is_empty()
    }

    #[inline]
    pub fn has_statement_hooks(&self) -> bool {
        !self.statement_hooks.is_empty()
    }

    #[inline]
    pub fn has_expression_hooks(&self) -> bool {
        !self.expression_hooks.is_empty()
    }

    #[inline]
    pub fn has_function_call_hooks(&self) -> bool {
        !self.function_call_hooks.is_empty()
    }

    #[inline]
    pub fn has_method_call_hooks(&self) -> bool {
        !self.method_call_hooks.is_empty()
    }

    #[inline]
    pub fn has_static_method_call_hooks(&self) -> bool {
        !self.static_method_call_hooks.is_empty()
    }

    #[inline]
    pub fn has_nullsafe_method_call_hooks(&self) -> bool {
        !self.nullsafe_method_call_hooks.is_empty()
    }

    #[inline]
    pub fn has_class_hooks(&self) -> bool {
        !self.class_hooks.is_empty()
    }

    #[inline]
    pub fn has_interface_hooks(&self) -> bool {
        !self.interface_hooks.is_empty()
    }

    #[inline]
    pub fn has_trait_hooks(&self) -> bool {
        !self.trait_hooks.is_empty()
    }

    #[inline]
    pub fn has_enum_hooks(&self) -> bool {
        !self.enum_hooks.is_empty()
    }

    #[inline]
    pub fn has_function_decl_hooks(&self) -> bool {
        !self.function_decl_hooks.is_empty()
    }

    #[inline]
    pub fn has_property_initialization_providers(&self) -> bool {
        !self.property_initialization_providers.is_empty()
    }

    #[inline]
    pub fn has_issue_filter_hooks(&self) -> bool {
        !self.issue_filter_hooks.is_empty()
    }

    #[inline]
    pub fn has_function_assertion_providers(&self) -> bool {
        !self.function_assertion_providers.is_empty()
    }

    #[inline]
    pub fn has_method_assertion_providers(&self) -> bool {
        !self.method_assertion_providers.is_empty()
    }

    #[inline]
    pub fn has_expression_throw_providers(&self) -> bool {
        !self.expression_throw_providers.is_empty()
    }

    #[inline]
    pub fn has_function_throw_providers(&self) -> bool {
        !self.function_throw_providers.is_empty()
    }

    #[inline]
    pub fn has_method_throw_providers(&self) -> bool {
        !self.method_throw_providers.is_empty()
    }

    pub fn before_program<'ast, 'arena>(
        &self,
        file: &File,
        program: &'ast Program<'arena>,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<HookAction> {
        for hook in &self.program_hooks {
            if let HookAction::Skip = hook.before_program(file, program, context)? {
                return Ok(HookAction::Skip);
            }
        }
        Ok(HookAction::Continue)
    }

    pub fn after_program<'ast, 'arena>(
        &self,
        file: &File,
        program: &'ast Program<'arena>,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<()> {
        for hook in &self.program_hooks {
            hook.after_program(file, program, context)?;
        }
        Ok(())
    }

    pub fn before_statement<'ast, 'arena>(
        &self,
        stmt: &'ast Statement<'arena>,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<HookAction> {
        for hook in &self.statement_hooks {
            if let HookAction::Skip = hook.before_statement(stmt, context)? {
                return Ok(HookAction::Skip);
            }
        }
        Ok(HookAction::Continue)
    }

    pub fn after_statement<'ast, 'arena>(
        &self,
        stmt: &'ast Statement<'arena>,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<()> {
        for hook in &self.statement_hooks {
            hook.after_statement(stmt, context)?;
        }
        Ok(())
    }

    pub fn before_expression<'ast, 'arena>(
        &self,
        expr: &'ast Expression<'arena>,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<ExpressionHookResult> {
        for hook in &self.expression_hooks {
            let result = hook.before_expression(expr, context)?;
            if result.should_skip() {
                return Ok(result);
            }
        }
        Ok(ExpressionHookResult::Continue)
    }

    pub fn after_expression<'ast, 'arena>(
        &self,
        expr: &'ast Expression<'arena>,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<()> {
        for hook in &self.expression_hooks {
            hook.after_expression(expr, context)?;
        }
        Ok(())
    }

    pub fn before_function_call<'ast, 'arena>(
        &self,
        call: &'ast FunctionCall<'arena>,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<ExpressionHookResult> {
        for hook in &self.function_call_hooks {
            let result = hook.before_function_call(call, context)?;
            if result.should_skip() {
                return Ok(result);
            }
        }
        Ok(ExpressionHookResult::Continue)
    }

    pub fn after_function_call<'ast, 'arena>(
        &self,
        call: &'ast FunctionCall<'arena>,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<()> {
        for hook in &self.function_call_hooks {
            hook.after_function_call(call, context)?;
        }
        Ok(())
    }

    pub fn before_method_call<'ast, 'arena>(
        &self,
        call: &'ast MethodCall<'arena>,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<ExpressionHookResult> {
        for hook in &self.method_call_hooks {
            let result = hook.before_method_call(call, context)?;
            if result.should_skip() {
                return Ok(result);
            }
        }
        Ok(ExpressionHookResult::Continue)
    }

    pub fn after_method_call<'ast, 'arena>(
        &self,
        call: &'ast MethodCall<'arena>,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<()> {
        for hook in &self.method_call_hooks {
            hook.after_method_call(call, context)?;
        }
        Ok(())
    }

    pub fn before_static_method_call<'ast, 'arena>(
        &self,
        call: &'ast StaticMethodCall<'arena>,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<ExpressionHookResult> {
        for hook in &self.static_method_call_hooks {
            let result = hook.before_static_method_call(call, context)?;
            if result.should_skip() {
                return Ok(result);
            }
        }
        Ok(ExpressionHookResult::Continue)
    }

    pub fn after_static_method_call<'ast, 'arena>(
        &self,
        call: &'ast StaticMethodCall<'arena>,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<()> {
        for hook in &self.static_method_call_hooks {
            hook.after_static_method_call(call, context)?;
        }
        Ok(())
    }

    pub fn before_nullsafe_method_call<'ast, 'arena>(
        &self,
        call: &'ast NullSafeMethodCall<'arena>,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<ExpressionHookResult> {
        for hook in &self.nullsafe_method_call_hooks {
            let result = hook.before_nullsafe_method_call(call, context)?;
            if result.should_skip() {
                return Ok(result);
            }
        }
        Ok(ExpressionHookResult::Continue)
    }

    pub fn after_nullsafe_method_call<'ast, 'arena>(
        &self,
        call: &'ast NullSafeMethodCall<'arena>,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<()> {
        for hook in &self.nullsafe_method_call_hooks {
            hook.after_nullsafe_method_call(call, context)?;
        }
        Ok(())
    }

    pub fn on_enter_class<'ast, 'arena>(
        &self,
        class: &'ast Class<'arena>,
        metadata: &ClassLikeMetadata,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<()> {
        for hook in &self.class_hooks {
            hook.on_enter_class(class, metadata, context)?;
        }
        Ok(())
    }

    pub fn on_leave_class<'ast, 'arena>(
        &self,
        class: &'ast Class<'arena>,
        metadata: &ClassLikeMetadata,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<()> {
        for hook in &self.class_hooks {
            hook.on_leave_class(class, metadata, context)?;
        }
        Ok(())
    }

    pub fn on_enter_interface<'ast, 'arena>(
        &self,
        interface: &'ast Interface<'arena>,
        metadata: &ClassLikeMetadata,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<()> {
        for hook in &self.interface_hooks {
            hook.on_enter_interface(interface, metadata, context)?;
        }
        Ok(())
    }

    pub fn on_leave_interface<'ast, 'arena>(
        &self,
        interface: &'ast Interface<'arena>,
        metadata: &ClassLikeMetadata,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<()> {
        for hook in &self.interface_hooks {
            hook.on_leave_interface(interface, metadata, context)?;
        }
        Ok(())
    }

    pub fn on_enter_trait<'ast, 'arena>(
        &self,
        trait_: &'ast Trait<'arena>,
        metadata: &ClassLikeMetadata,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<()> {
        for hook in &self.trait_hooks {
            hook.on_enter_trait(trait_, metadata, context)?;
        }
        Ok(())
    }

    pub fn on_leave_trait<'ast, 'arena>(
        &self,
        trait_: &'ast Trait<'arena>,
        metadata: &ClassLikeMetadata,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<()> {
        for hook in &self.trait_hooks {
            hook.on_leave_trait(trait_, metadata, context)?;
        }
        Ok(())
    }

    pub fn on_enter_enum<'ast, 'arena>(
        &self,
        enum_: &'ast Enum<'arena>,
        metadata: &ClassLikeMetadata,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<()> {
        for hook in &self.enum_hooks {
            hook.on_enter_enum(enum_, metadata, context)?;
        }
        Ok(())
    }

    pub fn on_leave_enum<'ast, 'arena>(
        &self,
        enum_: &'ast Enum<'arena>,
        metadata: &ClassLikeMetadata,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<()> {
        for hook in &self.enum_hooks {
            hook.on_leave_enum(enum_, metadata, context)?;
        }
        Ok(())
    }

    pub fn on_enter_function<'ast, 'arena>(
        &self,
        function: &'ast Function<'arena>,
        metadata: &FunctionLikeMetadata,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<()> {
        for hook in &self.function_decl_hooks {
            hook.on_enter_function(function, metadata, context)?;
        }
        Ok(())
    }

    pub fn on_leave_function<'ast, 'arena>(
        &self,
        function: &'ast Function<'arena>,
        metadata: &FunctionLikeMetadata,
        context: &mut HookContext<'_, '_>,
    ) -> PluginResult<()> {
        for hook in &self.function_decl_hooks {
            hook.on_leave_function(function, metadata, context)?;
        }
        Ok(())
    }

    fn get_function_provider_indices(&self, name: &str) -> Vec<usize> {
        let lower_name = ascii_lowercase_atom(name);
        let mut indices = Vec::new();

        if let Some(idxs) = self.function_exact.get(&lower_name) {
            indices.extend(idxs.iter().copied());
        }

        for (prefix, idx) in &self.function_prefix {
            if lower_name.as_str().starts_with(prefix.as_str()) && !indices.contains(idx) {
                indices.push(*idx);
            }
        }

        for (ns, idx) in &self.function_namespace {
            if lower_name.as_str().starts_with(ns.as_str()) && !indices.contains(idx) {
                indices.push(*idx);
            }
        }

        indices
    }

    fn get_method_provider_indices(&self, class_name: &str, method_name: &str) -> Vec<usize> {
        use mago_atom::concat_atom;
        let key =
            concat_atom!(ascii_lowercase_atom(class_name).as_str(), "::", ascii_lowercase_atom(method_name).as_str());
        let mut indices = Vec::new();

        if let Some(idxs) = self.method_exact.get(&key) {
            indices.extend(idxs.iter().copied());
        }

        for (targets, idx) in &self.method_wildcard {
            if !indices.contains(idx) {
                for target in targets {
                    if target.matches(class_name, method_name) {
                        indices.push(*idx);
                        break;
                    }
                }
            }
        }

        indices
    }

    pub fn get_function_like_return_type<'ctx, 'ast, 'arena>(
        &self,
        codebase: &'ctx CodebaseMetadata,
        block_context: &BlockContext<'ctx>,
        artifacts: &AnalysisArtifacts,
        function_like: &FunctionLikeIdentifier,
        invocation: &Invocation<'ctx, 'ast, 'arena>,
    ) -> Option<ProviderResult> {
        match function_like {
            FunctionLikeIdentifier::Function(name) => {
                Some(self.get_function_return_type(codebase, block_context, artifacts, name, invocation))
            }
            FunctionLikeIdentifier::Method(class_name, method_name) => Some(self.get_method_return_type(
                codebase,
                block_context,
                artifacts,
                class_name,
                method_name,
                invocation,
            )),
            _ => None,
        }
    }

    pub fn get_function_return_type<'ctx, 'ast, 'arena>(
        &self,
        codebase: &'ctx CodebaseMetadata,
        block_context: &BlockContext<'ctx>,
        artifacts: &AnalysisArtifacts,
        function_name: &str,
        invocation: &Invocation<'ctx, 'ast, 'arena>,
    ) -> ProviderResult {
        let indices = self.get_function_provider_indices(function_name);
        let mut all_issues = Vec::new();

        for idx in indices {
            let provider_context = ProviderContext::new(codebase, block_context, artifacts);
            let invocation_info = InvocationInfo::new(invocation);

            if let Some(ty) = self.function_providers[idx].get_return_type(&provider_context, &invocation_info) {
                all_issues.extend(provider_context.take_issues());
                return ProviderResult { return_type: Some(ty), issues: all_issues };
            }

            all_issues.extend(provider_context.take_issues());
        }

        ProviderResult { return_type: None, issues: all_issues }
    }

    pub fn get_method_return_type<'ctx, 'ast, 'arena>(
        &self,
        codebase: &'ctx CodebaseMetadata,
        block_context: &BlockContext<'ctx>,
        artifacts: &AnalysisArtifacts,
        class_name: &str,
        method_name: &str,
        invocation: &Invocation<'ctx, 'ast, 'arena>,
    ) -> ProviderResult {
        let indices = self.get_method_provider_indices(class_name, method_name);
        let mut all_issues = Vec::new();

        for idx in indices {
            let provider_context = ProviderContext::new(codebase, block_context, artifacts);
            let invocation_info = InvocationInfo::new(invocation);

            if let Some(ty) =
                self.method_providers[idx].get_return_type(&provider_context, class_name, method_name, &invocation_info)
            {
                all_issues.extend(provider_context.take_issues());
                return ProviderResult { return_type: Some(ty), issues: all_issues };
            }

            all_issues.extend(provider_context.take_issues());
        }

        ProviderResult { return_type: None, issues: all_issues }
    }

    #[inline]
    pub fn function_provider_count(&self) -> usize {
        self.function_providers.len()
    }

    #[inline]
    pub fn method_provider_count(&self) -> usize {
        self.method_providers.len()
    }

    /// Check if a property should be considered initialized by any registered provider.
    ///
    /// Returns `true` if any provider considers the property initialized.
    pub fn is_property_initialized(
        &self,
        class_metadata: &ClassLikeMetadata,
        property_metadata: &PropertyMetadata,
    ) -> bool {
        for provider in &self.property_initialization_providers {
            if provider.is_property_initialized(class_metadata, property_metadata) {
                return true;
            }
        }

        false
    }

    fn get_function_assertion_provider_indices(&self, name: &str) -> Vec<usize> {
        if self.function_assertion_exact.is_empty()
            && self.function_assertion_prefix.is_empty()
            && self.function_assertion_namespace.is_empty()
        {
            return Vec::new();
        }

        let lower_name = ascii_lowercase_atom(name);
        let mut indices = Vec::new();

        if let Some(idxs) = self.function_assertion_exact.get(&lower_name) {
            indices.extend(idxs.iter().copied());
        }

        for (prefix, idx) in &self.function_assertion_prefix {
            if lower_name.as_str().starts_with(prefix.as_str()) && !indices.contains(idx) {
                indices.push(*idx);
            }
        }

        for (ns, idx) in &self.function_assertion_namespace {
            if lower_name.as_str().starts_with(ns.as_str()) && !indices.contains(idx) {
                indices.push(*idx);
            }
        }

        indices
    }

    fn get_method_assertion_provider_indices(&self, class_name: &str, method_name: &str) -> Vec<usize> {
        if self.method_assertion_exact.is_empty() && self.method_assertion_wildcard.is_empty() {
            return Vec::new();
        }

        use mago_atom::concat_atom;
        let key =
            concat_atom!(ascii_lowercase_atom(class_name).as_str(), "::", ascii_lowercase_atom(method_name).as_str());
        let mut indices = Vec::new();

        if let Some(idxs) = self.method_assertion_exact.get(&key) {
            indices.extend(idxs.iter().copied());
        }

        for (targets, idx) in &self.method_assertion_wildcard {
            if !indices.contains(idx) {
                for target in targets {
                    if target.matches(class_name, method_name) {
                        indices.push(*idx);
                        break;
                    }
                }
            }
        }

        indices
    }

    pub fn get_function_like_assertions<'ctx, 'ast, 'arena>(
        &self,
        codebase: &'ctx CodebaseMetadata,
        block_context: &BlockContext<'ctx>,
        artifacts: &AnalysisArtifacts,
        function_like: &FunctionLikeIdentifier,
        invocation: &Invocation<'ctx, 'ast, 'arena>,
    ) -> Option<InvocationAssertions> {
        match function_like {
            FunctionLikeIdentifier::Function(name) => {
                self.get_function_assertions(codebase, block_context, artifacts, name, invocation)
            }
            FunctionLikeIdentifier::Method(class_name, method_name) => {
                self.get_method_assertions(codebase, block_context, artifacts, class_name, method_name, invocation)
            }
            _ => None,
        }
    }

    /// Get assertions for a function invocation from registered providers.
    pub fn get_function_assertions<'ctx, 'ast, 'arena>(
        &self,
        codebase: &'ctx CodebaseMetadata,
        block_context: &BlockContext<'ctx>,
        artifacts: &AnalysisArtifacts,
        function_name: &str,
        invocation: &Invocation<'ctx, 'ast, 'arena>,
    ) -> Option<InvocationAssertions> {
        if self.function_assertion_providers.is_empty() {
            return None;
        }

        let indices = self.get_function_assertion_provider_indices(function_name);

        for idx in indices {
            let provider_context = ProviderContext::new(codebase, block_context, artifacts);
            let invocation_info = InvocationInfo::new(invocation);

            if let Some(assertions) =
                self.function_assertion_providers[idx].get_assertions(&provider_context, &invocation_info)
                && !assertions.is_empty()
            {
                return Some(assertions);
            }
        }

        None
    }

    /// Get assertions for a method invocation from registered providers.
    pub fn get_method_assertions<'ctx, 'ast, 'arena>(
        &self,
        codebase: &'ctx CodebaseMetadata,
        block_context: &BlockContext<'ctx>,
        artifacts: &AnalysisArtifacts,
        class_name: &str,
        method_name: &str,
        invocation: &Invocation<'ctx, 'ast, 'arena>,
    ) -> Option<InvocationAssertions> {
        if self.method_assertion_providers.is_empty() {
            return None;
        }

        let indices = self.get_method_assertion_provider_indices(class_name, method_name);

        for idx in indices {
            let provider_context = ProviderContext::new(codebase, block_context, artifacts);
            let invocation_info = InvocationInfo::new(invocation);

            if let Some(assertions) = self.method_assertion_providers[idx].get_assertions(
                &provider_context,
                class_name,
                method_name,
                &invocation_info,
            ) && !assertions.is_empty()
            {
                return Some(assertions);
            }
        }

        None
    }

    fn get_function_throw_provider_indices(&self, name: &str) -> Vec<usize> {
        if self.function_throw_exact.is_empty()
            && self.function_throw_prefix.is_empty()
            && self.function_throw_namespace.is_empty()
        {
            return Vec::new();
        }

        let lower_name = ascii_lowercase_atom(name);
        let mut indices = Vec::new();

        if let Some(idxs) = self.function_throw_exact.get(&lower_name) {
            indices.extend(idxs.iter().copied());
        }

        for (prefix, idx) in &self.function_throw_prefix {
            if lower_name.as_str().starts_with(prefix.as_str()) && !indices.contains(idx) {
                indices.push(*idx);
            }
        }

        for (ns, idx) in &self.function_throw_namespace {
            if lower_name.as_str().starts_with(ns.as_str()) && !indices.contains(idx) {
                indices.push(*idx);
            }
        }

        indices
    }

    fn get_method_throw_provider_indices(&self, class_name: &str, method_name: &str) -> Vec<usize> {
        if self.method_throw_providers.is_empty()
            && self.method_throw_exact.is_empty()
            && self.method_throw_wildcard.is_empty()
        {
            return Vec::new();
        }

        use mago_atom::concat_atom;
        let key =
            concat_atom!(ascii_lowercase_atom(class_name).as_str(), "::", ascii_lowercase_atom(method_name).as_str());
        let mut indices = Vec::new();

        if let Some(idxs) = self.method_throw_exact.get(&key) {
            indices.extend(idxs.iter().copied());
        }

        for (targets, idx) in &self.method_throw_wildcard {
            if !indices.contains(idx) {
                for target in targets {
                    if target.matches(class_name, method_name) {
                        indices.push(*idx);
                        break;
                    }
                }
            }
        }

        indices
    }

    /// Get thrown exception class names for an expression from registered providers.
    pub fn get_expression_thrown_exceptions<'ctx, 'ast, 'arena>(
        &self,
        codebase: &'ctx CodebaseMetadata,
        block_context: &BlockContext<'ctx>,
        artifacts: &AnalysisArtifacts,
        expression: &'ast mago_syntax::ast::Expression<'arena>,
    ) -> AtomSet {
        let mut exceptions = AtomSet::default();

        for provider in &self.expression_throw_providers {
            let provider_context = ProviderContext::new(codebase, block_context, artifacts);
            exceptions.extend(provider.get_thrown_exceptions(&provider_context, expression));
        }

        exceptions
    }

    /// Get thrown exception class names for a function invocation from registered providers.
    pub fn get_function_thrown_exceptions<'ctx, 'ast, 'arena>(
        &self,
        codebase: &'ctx CodebaseMetadata,
        block_context: &BlockContext<'ctx>,
        artifacts: &AnalysisArtifacts,
        function_name: &str,
        invocation: &Invocation<'ctx, 'ast, 'arena>,
    ) -> AtomSet {
        let mut exceptions = AtomSet::default();
        let indices = self.get_function_throw_provider_indices(function_name);

        for idx in indices {
            let provider_context = ProviderContext::new(codebase, block_context, artifacts);
            let invocation_info = InvocationInfo::new(invocation);
            exceptions
                .extend(self.function_throw_providers[idx].get_thrown_exceptions(&provider_context, &invocation_info));
        }

        exceptions
    }

    /// Get thrown exception class names for a method invocation from registered providers.
    pub fn get_method_thrown_exceptions<'ctx, 'ast, 'arena>(
        &self,
        codebase: &'ctx CodebaseMetadata,
        block_context: &BlockContext<'ctx>,
        artifacts: &AnalysisArtifacts,
        class_name: &str,
        method_name: &str,
        invocation: &Invocation<'ctx, 'ast, 'arena>,
    ) -> AtomSet {
        let mut exceptions = AtomSet::default();
        let indices = self.get_method_throw_provider_indices(class_name, method_name);

        for idx in indices {
            let provider_context = ProviderContext::new(codebase, block_context, artifacts);
            let invocation_info = InvocationInfo::new(invocation);
            exceptions.extend(self.method_throw_providers[idx].get_thrown_exceptions(
                &provider_context,
                class_name,
                method_name,
                &invocation_info,
            ));
        }

        exceptions
    }

    /// Filter issues through all registered issue filter hooks.
    ///
    /// Returns a new `IssueCollection` with filtered issues.
    pub fn filter_issues(&self, file: &File, issues: IssueCollection) -> IssueCollection {
        if self.issue_filter_hooks.is_empty() {
            return issues;
        }

        let mut filtered = IssueCollection::default();

        for issue in issues {
            let mut keep = true;
            for hook in &self.issue_filter_hooks {
                if let Ok(IssueFilterDecision::Remove) = hook.filter_issue(file, &issue) {
                    keep = false;
                    break;
                }
            }

            if keep {
                filtered.push(issue);
            }
        }

        filtered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::provider::Provider;
    use crate::plugin::provider::ProviderMeta;

    static TEST_META: ProviderMeta = ProviderMeta::new("test::provider", "Test Provider", "A test provider");

    struct TestFunctionProvider;

    impl Provider for TestFunctionProvider {
        fn meta() -> &'static ProviderMeta {
            &TEST_META
        }
    }

    impl FunctionReturnTypeProvider for TestFunctionProvider {
        fn targets() -> FunctionTarget {
            FunctionTarget::Exact("test_func")
        }

        fn get_return_type(
            &self,
            _context: &ProviderContext<'_, '_, '_>,
            _invocation: &InvocationInfo<'_, '_, '_>,
        ) -> Option<TUnion> {
            None
        }
    }

    #[test]
    fn test_register_function_provider() {
        let mut registry = PluginRegistry::new();
        registry.register_function_provider(TestFunctionProvider);

        assert_eq!(registry.function_provider_count(), 1);
        let indices = registry.get_function_provider_indices("test_func");
        assert_eq!(indices.len(), 1);
    }

    #[test]
    fn test_function_exact_match() {
        let mut registry = PluginRegistry::new();
        registry.register_function_provider(TestFunctionProvider);

        let indices = registry.get_function_provider_indices("test_func");
        assert_eq!(indices.len(), 1);

        let indices = registry.get_function_provider_indices("TEST_FUNC");
        assert_eq!(indices.len(), 1);

        let indices = registry.get_function_provider_indices("other_func");
        assert!(indices.is_empty());
    }
}
