use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_flags::U8Flags;
use mago_flags::U16Flags;
use mago_hir::ir::item::annotation::ItemAnnotation;
use mago_hir::ir::item::annotation::effect::AssertAnnotation;
use mago_hir::ir::item::annotation::effect::AssertAnnotationPatternKind;
use mago_hir::ir::item::annotation::effect::AssertAnnotationTargetKind;
use mago_hir::ir::item::annotation::generics::Variance as HirVariance;
use mago_hir::ir::item::annotation::parameter::ParameterOutAnnotation;
use mago_hir::ir::item::attribute::Attribute;
use mago_hir::ir::item::expression::arrow_function::ArrowFunction;
use mago_hir::ir::item::expression::arrow_function::ArrowFunctionFlag as HirArrowFunctionFlag;
use mago_hir::ir::item::expression::closure::Closure;
use mago_hir::ir::item::expression::closure::ClosureFlag as HirClosureFlag;
use mago_hir::ir::item::parameter::Parameter;
use mago_hir::ir::item::parameter::ParameterFlag as HirParameterFlag;
use mago_hir::ir::item::statement::constant::Constant;
use mago_hir::ir::item::statement::function::Function;
use mago_hir::ir::item::statement::function::FunctionFlag as HirFunctionFlag;
use mago_hir::ir::r#type::Type as HirType;
use mago_hir::ir::r#type::annotation::TypeAnnotation;
use mago_hir::ir::variable::DirectVariable;
use mago_php_version::PHPVersionRange;

use crate::assertion::Assertion;
use crate::id::SymbolId;
use crate::path::Path;
use crate::symbol::constant::ConstantSymbol;
use crate::symbol::function_like::arrow_function::ArrowFunctionFlag;
use crate::symbol::function_like::arrow_function::ArrowFunctionSymbol;
use crate::symbol::function_like::closure::ClosureFlag;
use crate::symbol::function_like::closure::ClosureSymbol;
use crate::symbol::function_like::function::FunctionFlag;
use crate::symbol::function_like::function::FunctionSymbol;
use crate::symbol::function_like::part::assertion::FunctionLikeAssertion;
use crate::symbol::function_like::part::assertion::FunctionLikeAssertionFlag;
use crate::symbol::function_like::part::assertion::FunctionLikeAssertionTarget;
use crate::symbol::function_like::part::parameter::SignatureParameter;
use crate::symbol::function_like::part::parameter::SignatureParameterFlag;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::generic::GenericParameter;
use crate::symbol::part::generic::Variance;
use crate::symbol::part::origin::Origin;
use crate::symbol::part::ty::TypeSlot;
use crate::ty::Type;
use crate::ty::builder::TypeBuilder;
use crate::var::Var;

/// Lowers declared HIR definitions into their resolved oracle symbols. Holds the
/// output arena and the [`TypeBuilder`] every lowered type is interned through.
pub(crate) struct Lowerer<'builder, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub arena: &'arena A,
    pub builder: &'builder mut TypeBuilder<'scratch, 'arena, S, A>,
}

impl<'arena, S, A> Lowerer<'_, '_, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    /// The PHP-version availability of a declaration; empty ranges mean
    /// unconstrained.
    pub(crate) fn constraint(&self, ranges: &'arena [PHPVersionRange]) -> SymbolConstraint<'arena> {
        SymbolConstraint { ranges }
    }

    /// Lowers each applied attribute to its name path.
    pub(crate) fn attributes<I, St, Ex>(
        &self,
        attributes: &[Attribute<'arena, I, St, Ex>],
    ) -> &'arena [AppliedAttribute<'arena>] {
        let arena = self.arena;
        arena.alloc_slice_fill_iter(attributes.iter().map(|attribute| AppliedAttribute {
            span: attribute.span,
            name: Path::class_like(arena, attribute.class.value),
        }))
    }

    /// Lowers the globals a function-like accesses to interned [`Var`]s.
    pub(crate) fn accessed_globals(&self, globals: &[DirectVariable<'arena>]) -> &'arena [Var<'arena>] {
        self.arena.alloc_slice_fill_iter(globals.iter().map(|global| Var::new(global.name)))
    }

    /// Lowers a function-like's parameter list into signature parameters owned by
    /// `owner` (whose fully-qualified `owner_name` keys each parameter's path).
    pub(crate) fn function_like_parameters<I, St, Ex>(
        &mut self,
        owner_name: &'arena [u8],
        parameters: &[Parameter<'arena, I, St, Ex>],
        annotation: Option<&ItemAnnotation<'arena, I, St, Ex>>,
        origin: Origin,
    ) -> &'arena [SignatureParameter<'arena>] {
        let owner = SymbolId::function_like(owner_name);
        let parameter_outs = annotation.map_or(&[][..], |annotation| annotation.parameter_outs);
        let arena = self.arena;
        arena.alloc_slice_fill_iter(parameters.iter().map(|parameter| {
            let mut flags = U8Flags::<SignatureParameterFlag>::empty();
            if parameter.flags.contains(HirParameterFlag::ByReference) {
                flags = flags.with(SignatureParameterFlag::ByReference);
            }
            if parameter.flags.contains(HirParameterFlag::IsVariadic) {
                flags = flags.with(SignatureParameterFlag::Variadic);
            }
            if parameter.default_value.is_some() {
                flags = flags.with(SignatureParameterFlag::HasDefault);
            }

            let attributes = self.attributes(parameter.attributes);
            let ty = self.type_slot_annotated(
                parameter.r#type,
                parameter.annotation.map(|annotation| annotation.type_annotation),
            );
            let out_ty = self.parameter_out_slot(parameter_outs, parameter.variable.name);
            let default_ty = self.default_type_slot(parameter.default_value);

            SignatureParameter {
                span: parameter.span,
                defining_symbol: owner,
                path: Path::function_like_parameter(arena, owner_name, parameter.variable.name),
                attributes,
                flags,
                constraint: self.constraint(parameter.version_constraint),
                ty,
                out_ty,
                default_ty,
                origin,
            }
        }))
    }

    /// The `@self-out` type a method declares for `$this` after the call, lowered
    /// into a slot's `annotation` channel; `None` when not declared.
    pub(crate) fn self_out<I, St, Ex>(
        &mut self,
        annotation: Option<&ItemAnnotation<'arena, I, St, Ex>>,
    ) -> Option<TypeSlot<'arena>> {
        let self_out = annotation?.self_out.first()?;
        let mut slot = TypeSlot::new();
        slot.annotation = self.lower_type_annotation(self_out.r#type);

        Some(slot)
    }

    /// The indices of the parameters named in a `@pure-unless-callable-impure`
    /// annotation: the function is pure unless one of those callable parameters
    /// is itself impure.
    pub(crate) fn pure_unless_impure_params<I, St, Ex>(
        &self,
        parameters: &[Parameter<'arena, I, St, Ex>],
        annotation: Option<&ItemAnnotation<'arena, I, St, Ex>>,
    ) -> &'arena [u32] {
        let Some(annotation) = annotation else {
            return &[];
        };
        if annotation.pure_unless_callable_impure.is_empty() {
            return &[];
        }

        self.arena.alloc_slice_fill_iter(parameters.iter().enumerate().filter_map(|(index, parameter)| {
            annotation
                .pure_unless_callable_impure
                .iter()
                .any(|variable| variable.name == parameter.variable.name)
                .then_some(index as u32)
        }))
    }

    /// Infers a parameter's default-value type into a slot's `inferred` channel;
    /// an empty slot when the parameter has no default or it cannot be inferred.
    pub(crate) fn default_type_slot<I, St, Ex>(
        &mut self,
        default_value: Option<&mago_hir::ir::expression::Expression<'arena, I, St, Ex>>,
    ) -> TypeSlot<'arena> {
        let mut slot = TypeSlot::new();
        slot.inferred = default_value.and_then(|value| self.infer(value));

        slot
    }

    /// The `@param-out` type declared for the parameter named `variable`, lowered
    /// into a slot's `annotation` channel; an empty slot when none is declared.
    pub(crate) fn parameter_out_slot(
        &mut self,
        parameter_outs: &[ParameterOutAnnotation<'arena>],
        variable: &[u8],
    ) -> TypeSlot<'arena> {
        let mut slot = TypeSlot::new();
        slot.annotation = parameter_outs
            .iter()
            .find(|out| out.variable.name == variable)
            .and_then(|out| self.lower_type_annotation(out.r#type));

        slot
    }

    /// Lowers a native type hint into a [`TypeSlot`]'s `hint` channel.
    pub(crate) fn type_slot(&mut self, hint: Option<&HirType<'arena>>) -> TypeSlot<'arena> {
        let mut slot = TypeSlot::new();
        slot.hint = hint.and_then(|hint| self.lower_hir_type(hint));

        slot
    }

    /// Lowers a native type hint into the `hint` channel and a phpdoc type
    /// annotation into the `annotation` channel of one [`TypeSlot`].
    pub(crate) fn type_slot_annotated(
        &mut self,
        hint: Option<&HirType<'arena>>,
        annotation: Option<&TypeAnnotation<'arena>>,
    ) -> TypeSlot<'arena> {
        let mut slot = TypeSlot::new();
        slot.hint = hint.and_then(|hint| self.lower_hir_type(hint));
        slot.annotation = annotation.and_then(|annotation| self.lower_type_annotation(annotation));

        slot
    }

    /// Lowers a free function declaration into its [`FunctionSymbol`].
    pub(crate) fn function<I, St, Ex>(
        &mut self,
        function: &Function<'arena, I, St, Ex>,
        origin: Origin,
    ) -> FunctionSymbol<'arena> {
        let mut flags = U16Flags::<FunctionFlag>::empty();
        if function.flags.contains(HirFunctionFlag::Yields) {
            flags = flags.with(FunctionFlag::HasYield);
        }
        if function.flags.contains(HirFunctionFlag::Throws) {
            flags = flags.with(FunctionFlag::HasThrow);
        }
        if function.flags.contains(HirFunctionFlag::ReturnsByReference) {
            flags = flags.with(FunctionFlag::ReturnsByReference);
        }
        if function.flags.contains(HirFunctionFlag::AssertionsInferred) {
            flags = flags.with(FunctionFlag::AssertionsInferred);
        }
        flags = flags.union(crate::linker::tags::function_flags(crate::linker::tags::tags_of(function.annotation)));

        let attributes = self.attributes(function.attributes);
        let generics = self.generics(SymbolId::function_like(function.name.value), function.annotation);
        let params = self.function_like_parameters(
            function.name.value,
            function.parameters.as_slice(),
            function.annotation,
            origin,
        );
        let ret = self.type_slot_annotated(function.return_type, return_annotation(function.annotation));
        let throws = self.throws(function.annotation);
        let assertions = self.assertions(function.annotation);
        let pure_unless_impure_params =
            self.pure_unless_impure_params(function.parameters.as_slice(), function.annotation);
        let accessed_globals = self.accessed_globals(function.direct_accessed_globals);

        FunctionSymbol {
            span: function.span,
            name: Path::function_like(self.arena, function.name.value),
            flags,
            constraint: self.constraint(function.version_constraint),
            attributes,
            generics,
            params,
            ret,
            throws,
            assertions,
            pure_unless_impure_params,
            accessed_globals,
            origin,
        }
    }

    /// Lowers a closure declaration into its [`ClosureSymbol`].
    pub(crate) fn closure<I, St, Ex>(
        &mut self,
        closure: &Closure<'arena, I, St, Ex>,
        origin: Origin,
    ) -> ClosureSymbol<'arena> {
        let mut flags = U16Flags::<ClosureFlag>::empty();
        if closure.flags.contains(HirClosureFlag::Yields) {
            flags = flags.with(ClosureFlag::HasYield);
        }
        if closure.flags.contains(HirClosureFlag::Throws) {
            flags = flags.with(ClosureFlag::HasThrow);
        }
        if closure.flags.contains(HirClosureFlag::ReturnsByReference) {
            flags = flags.with(ClosureFlag::ReturnsByReference);
        }
        if closure.flags.contains(HirClosureFlag::AssertionsInferred) {
            flags = flags.with(ClosureFlag::AssertionsInferred);
        }

        let attributes = self.attributes(closure.attributes);
        let generics = self.generics(SymbolId::function_like(closure.name), closure.annotation);
        let params =
            self.function_like_parameters(closure.name, closure.parameters.as_slice(), closure.annotation, origin);
        let ret = self.type_slot_annotated(closure.return_type, return_annotation(closure.annotation));
        let throws = self.throws(closure.annotation);
        let assertions = self.assertions(closure.annotation);
        let pure_unless_impure_params =
            self.pure_unless_impure_params(closure.parameters.as_slice(), closure.annotation);
        let accessed_globals = self.accessed_globals(closure.direct_accessed_globals);

        ClosureSymbol {
            span: closure.span,
            name: Path::function_like(self.arena, closure.name),
            flags,
            constraint: SymbolConstraint::unconstrained(),
            attributes,
            generics,
            params,
            ret,
            throws,
            assertions,
            pure_unless_impure_params,
            accessed_globals,
            origin,
        }
    }

    /// Lowers an arrow-function declaration into its [`ArrowFunctionSymbol`].
    pub(crate) fn arrow_function<I, St, Ex>(
        &mut self,
        arrow_function: &ArrowFunction<'arena, I, St, Ex>,
        origin: Origin,
    ) -> ArrowFunctionSymbol<'arena> {
        let mut flags = U16Flags::<ArrowFunctionFlag>::empty();
        if arrow_function.flags.contains(HirArrowFunctionFlag::Yields) {
            flags = flags.with(ArrowFunctionFlag::HasYield);
        }
        if arrow_function.flags.contains(HirArrowFunctionFlag::Throws) {
            flags = flags.with(ArrowFunctionFlag::HasThrow);
        }
        if arrow_function.flags.contains(HirArrowFunctionFlag::ReturnsByReference) {
            flags = flags.with(ArrowFunctionFlag::ReturnsByReference);
        }
        if arrow_function.flags.contains(HirArrowFunctionFlag::AssertionsInferred) {
            flags = flags.with(ArrowFunctionFlag::AssertionsInferred);
        }

        let attributes = self.attributes(arrow_function.attributes);
        let generics = self.generics(SymbolId::function_like(arrow_function.name), arrow_function.annotation);
        let params = self.function_like_parameters(
            arrow_function.name,
            arrow_function.parameters.as_slice(),
            arrow_function.annotation,
            origin,
        );
        let ret = self.type_slot_annotated(arrow_function.return_type, return_annotation(arrow_function.annotation));
        let throws = self.throws(arrow_function.annotation);
        let assertions = self.assertions(arrow_function.annotation);
        let pure_unless_impure_params =
            self.pure_unless_impure_params(arrow_function.parameters.as_slice(), arrow_function.annotation);

        ArrowFunctionSymbol {
            span: arrow_function.span,
            name: Path::function_like(self.arena, arrow_function.name),
            flags,
            constraint: SymbolConstraint::unconstrained(),
            attributes,
            generics,
            params,
            ret,
            throws,
            assertions,
            pure_unless_impure_params,
            origin,
        }
    }

    /// Lowers a global constant declaration into its [`ConstantSymbol`].
    pub(crate) fn constant<I, St, Ex>(
        &mut self,
        constant: &Constant<'arena, I, St, Ex>,
        origin: Origin,
    ) -> ConstantSymbol<'arena> {
        let attributes = self.attributes(constant.attributes);
        let mut ty = TypeSlot::new();
        ty.inferred = self.infer(constant.value);

        ConstantSymbol {
            span: constant.span,
            name: Path::constant(self.arena, constant.name.value),
            attributes,
            flags: U8Flags::empty(),
            constraint: self.constraint(constant.version_constraint),
            ty,
            origin,
        }
    }

    /// Lowers a HIR type into an oracle [`Type`]. Completed by the type-bridge
    /// layer; the seam keeps every call site wired.
    pub(crate) fn lower_hir_type(&mut self, hir: &HirType<'arena>) -> Option<Type<'arena>> {
        crate::linker::types::lower_hir_type(self.builder, hir)
    }

    /// Lowers the `@template` parameters declared on a class-like or function-like
    /// into generic parameters owned by `defining_entity`. A parameter with no
    /// declared bound is constrained to `mixed`.
    pub(crate) fn generics<I, St, Ex>(
        &mut self,
        defining_entity: crate::id::SymbolId,
        annotation: Option<&ItemAnnotation<'arena, I, St, Ex>>,
    ) -> &'arena [GenericParameter<'arena>] {
        match annotation {
            Some(annotation) => self.generic_parameters(defining_entity, annotation.type_parameters),
            None => &[],
        }
    }

    /// Lowers a list of `@template` parameter annotations into generic parameters
    /// owned by `defining_entity`. A parameter with no declared bound is
    /// constrained to `mixed`.
    pub(crate) fn generic_parameters(
        &mut self,
        defining_entity: crate::id::SymbolId,
        parameters: &[mago_hir::ir::item::annotation::generics::TypeParameterAnnotation<'arena>],
    ) -> &'arena [GenericParameter<'arena>] {
        let arena = self.arena;
        arena.alloc_slice_fill_iter(parameters.iter().map(|parameter| {
            let constraint = parameter
                .bound
                .and_then(|bound| self.lower_type_annotation(bound))
                .unwrap_or(crate::ty::well_known::TYPE_MIXED);

            GenericParameter {
                span: parameter.span,
                name: parameter.name.value,
                defining_entity,
                variance: match parameter.variance {
                    HirVariance::Invariant => Variance::Invariant,
                    HirVariance::Covariant => Variance::Covariant,
                    HirVariance::Contravariant => Variance::Contravariant,
                },
                constraint,
                default: parameter.default.and_then(|default| self.lower_type_annotation(default)),
            }
        }))
    }

    /// Lowers a phpdoc type annotation into an oracle [`Type`]. Completed by the
    /// type-bridge layer; the seam keeps every call site wired.
    pub(crate) fn lower_type_annotation(&mut self, annotation: &TypeAnnotation<'arena>) -> Option<Type<'arena>> {
        crate::linker::types::lower_type_annotation(self.builder, annotation)
    }

    /// Lowers the `@throws` types declared in a docblock into the exception
    /// types a function-like may throw.
    pub(crate) fn throws<I, St, Ex>(
        &mut self,
        annotation: Option<&ItemAnnotation<'arena, I, St, Ex>>,
    ) -> &'arena [Type<'arena>] {
        let Some(annotation) = annotation else {
            return &[];
        };

        let arena = self.arena;
        arena.alloc_slice_fill_iter(
            annotation.throws.iter().filter_map(|throws| self.lower_type_annotation(throws.r#type)),
        )
    }

    /// Lowers the `@template`-`of`/`as` where-constraints declared on a method or
    /// function into resolved constraints.
    pub(crate) fn where_constraints<I, St, Ex>(
        &mut self,
        annotation: Option<&ItemAnnotation<'arena, I, St, Ex>>,
    ) -> &'arena [crate::symbol::part::generic::WhereConstraint<'arena>] {
        let Some(annotation) = annotation else {
            return &[];
        };

        let arena = self.arena;
        arena.alloc_slice_fill_iter(annotation.where_constraints.iter().map(|constraint| {
            crate::symbol::part::generic::WhereConstraint {
                span: constraint.span,
                parameter: constraint.type_parameter.value,
                ty: self.lower_type_annotation(constraint.constraint).unwrap_or(crate::ty::well_known::TYPE_MIXED),
            }
        }))
    }

    /// Lowers the `@assert` / `@assert-if-true` / `@assert-if-false` annotations
    /// declared on a function-like into resolved assertions.
    pub(crate) fn assertions<I, St, Ex>(
        &mut self,
        annotation: Option<&ItemAnnotation<'arena, I, St, Ex>>,
    ) -> &'arena [FunctionLikeAssertion<'arena>] {
        use crate::symbol::function_like::part::assertion::FunctionLikeAssertionFlag;

        let Some(annotation) = annotation else {
            return &[];
        };

        let scratch = self.builder.scratch();
        let mut assertions = Vec::new_in(scratch);
        for assertion in annotation.asserts {
            let lowered = self.assertion(assertion, U8Flags::empty());
            assertions.push(lowered);
        }
        for assertion in annotation.asserts_if_true {
            let lowered = self.assertion(assertion, U8Flags::empty().with(FunctionLikeAssertionFlag::IfTrue));
            assertions.push(lowered);
        }
        for assertion in annotation.asserts_if_false {
            let lowered = self.assertion(assertion, U8Flags::empty().with(FunctionLikeAssertionFlag::IfFalse));
            assertions.push(lowered);
        }

        self.arena.alloc_slice_fill_iter(assertions)
    }

    /// Lowers one `@assert` annotation into a [`FunctionLikeAssertion`] with the
    /// given condition flags (if-true / if-false / unconditional).
    fn assertion(
        &mut self,
        annotation: &AssertAnnotation<'arena>,
        flags: U8Flags<FunctionLikeAssertionFlag>,
    ) -> FunctionLikeAssertion<'arena> {
        let target = match annotation.target.kind {
            AssertAnnotationTargetKind::Variable(variable) => {
                FunctionLikeAssertionTarget::Parameter(Var::new(variable.name))
            }
            AssertAnnotationTargetKind::Method(variable, name) => {
                FunctionLikeAssertionTarget::Method(Var::new(variable.name), name.value)
            }
            AssertAnnotationTargetKind::Property(variable, name) => {
                FunctionLikeAssertionTarget::Property(Var::new(variable.name), name.value)
            }
        };

        let assertion = self.assertion_value(&annotation.pattern.kind, annotation.negated, annotation.equality);

        FunctionLikeAssertion { span: annotation.span, flags, target, assertion }
    }

    /// Resolves an `@assert` pattern into an [`Assertion`], honouring negation
    /// and the loose-vs-identity (`equality`) distinction.
    fn assertion_value(
        &mut self,
        pattern: &AssertAnnotationPatternKind<'arena>,
        negated: bool,
        equality: bool,
    ) -> Assertion<'arena> {
        match pattern {
            AssertAnnotationPatternKind::Truthy => {
                if negated {
                    Assertion::Falsy
                } else {
                    Assertion::Truthy
                }
            }
            AssertAnnotationPatternKind::Falsy => {
                if negated {
                    Assertion::Truthy
                } else {
                    Assertion::Falsy
                }
            }
            AssertAnnotationPatternKind::NonEmpty => {
                if negated {
                    Assertion::Empty
                } else {
                    Assertion::NonEmpty
                }
            }
            AssertAnnotationPatternKind::Type(annotation) => {
                let Some(atom) = self.lower_type_annotation(annotation).and_then(|ty| ty.atoms.first().copied()) else {
                    return Assertion::Any;
                };

                match (equality, negated) {
                    (true, false) => Assertion::IsIdentical(atom),
                    (true, true) => Assertion::IsNotIdentical(atom),
                    (false, false) => Assertion::IsType(atom),
                    (false, true) => Assertion::IsNotType(atom),
                }
            }
        }
    }
}

/// The `@return` type annotation declared in a function-like's docblock, if any.
pub(crate) fn return_annotation<'arena, I, St, Ex>(
    annotation: Option<&ItemAnnotation<'arena, I, St, Ex>>,
) -> Option<&'arena TypeAnnotation<'arena>> {
    annotation.and_then(|annotation| annotation.return_type.first())
}
