use mago_allocator::Arena;
use mago_flags::U8Flags;
use mago_flags::U16Flags;
use mago_flags::U32Flags;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::item::annotation::ItemAnnotation;
use mago_hir::ir::item::annotation::member::MethodAnnotation;
use mago_hir::ir::item::annotation::member::PropertyAnnotation;
use mago_hir::ir::item::annotation::member::PropertyAnnotationKind;
use mago_hir::ir::item::member::MemberItem;
use mago_hir::ir::item::member::MemberItemKind;
use mago_hir::ir::item::member::constant::ClassLikeConstant;
use mago_hir::ir::item::member::enum_case::EnumCase;
use mago_hir::ir::item::member::hook::Hook;
use mago_hir::ir::item::member::hook::HookFlag;
use mago_hir::ir::item::member::method::Method;
use mago_hir::ir::item::member::method::MethodFlag as HirMethodFlag;
use mago_hir::ir::item::member::property::HookedProperty;
use mago_hir::ir::item::member::property::Property;
use mago_hir::ir::item::modifier::Modifier;
use mago_hir::ir::item::modifier::ModifierKind;
use mago_hir::ir::item::modifier::Visibility as HirVisibility;
use mago_hir::ir::item::modifier::VisibilityKind;
use mago_hir::ir::item::parameter::Parameter;
use mago_hir::ir::item::parameter::ParameterFlag as HirParameterFlag;
use mago_hir::ir::literal::LiteralKind;
use mago_hir::ir::r#type::annotation::TypeAnnotation;

use crate::id::SymbolId;
use crate::linker::index::sorted_offsets;
use crate::linker::lower::Lowerer;
use crate::linker::lower::return_annotation;
use crate::path::Path;
use crate::symbol::class_like::part::constant::ClassLikeConstantFlag;
use crate::symbol::class_like::part::constant::ClassLikeConstantMember;
use crate::symbol::class_like::part::constant::ClassLikeConstantMemberList;
use crate::symbol::class_like::part::enum_case::EnumCaseFlag;
use crate::symbol::class_like::part::enum_case::EnumCaseMember;
use crate::symbol::class_like::part::enum_case::EnumCaseMemberList;
use crate::symbol::class_like::part::inheritance::InheritedType;
use crate::symbol::class_like::part::inheritance::InheritedTypeList;
use crate::symbol::class_like::part::inheritance::Provenance;
use crate::symbol::class_like::part::method::MethodFlag;
use crate::symbol::class_like::part::method::MethodMember;
use crate::symbol::class_like::part::method::MethodMemberList;
use crate::symbol::class_like::part::property::PropertyFlag;
use crate::symbol::class_like::part::property::PropertyMember;
use crate::symbol::class_like::part::property::PropertyMemberList;
use crate::symbol::class_like::part::property_hook::HookKind;
use crate::symbol::class_like::part::property_hook::PropertyHookFlag;
use crate::symbol::class_like::part::property_hook::PropertyHookMember;
use crate::symbol::class_like::part::visibility::ReadWriteVisibility;
use crate::symbol::class_like::part::visibility::Visibility;
use crate::symbol::function_like::part::parameter::SignatureParameter;
use crate::symbol::function_like::part::parameter::SignatureParameterFlag;
use crate::symbol::part::origin::Origin;
use crate::symbol::part::ty::TypeSlot;
use crate::ty::Atom;

/// The own (non-inherited) members of one class-like, plus the direct trait-use
/// edges discovered while walking them.
pub(crate) struct OwnMembers<'arena> {
    pub constants: ClassLikeConstantMemberList<'arena>,
    pub properties: PropertyMemberList<'arena>,
    pub methods: MethodMemberList<'arena>,
    pub cases: EnumCaseMemberList<'arena>,
    pub uses: InheritedTypeList<'arena>,
}

impl<'arena, S, A> Lowerer<'_, '_, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    /// Lowers the own members of a class-like named `class_name` into resolved
    /// member lists. Inherited members are merged in later by the resolve pass;
    /// every member's `defining_symbol` is this class.
    pub(crate) fn own_members<I, St, Ex>(
        &mut self,
        class_name: &'arena [u8],
        members: &[MemberItem<'arena, I, St, Ex>],
        annotation: Option<&ItemAnnotation<'arena, I, St, Ex>>,
        origin: Origin,
    ) -> OwnMembers<'arena> {
        let owner = SymbolId::class_like(class_name);

        let mut constants = self.builder.scratch_vec();
        let mut properties = self.builder.scratch_vec();
        let mut methods = self.builder.scratch_vec();
        let mut cases = self.builder.scratch_vec();
        let mut uses = self.builder.scratch_vec();

        for member in members {
            match &member.kind {
                MemberItemKind::Method(method) => {
                    methods.push(self.method(class_name, owner, method, origin));
                }
                MemberItemKind::Property(property) => {
                    properties.push(self.property(class_name, owner, property, origin));
                }
                MemberItemKind::HookedProperty(property) => {
                    properties.push(self.hooked_property(class_name, owner, property, origin));
                }
                MemberItemKind::Constant(constant) => {
                    constants.push(self.constant_member(class_name, owner, constant, origin));
                }
                MemberItemKind::EnumCase(case) => {
                    cases.push(self.enum_case(class_name, owner, case, origin));
                }
                MemberItemKind::TraitUse(trait_use) => {
                    for target in trait_use.traits {
                        uses.push(InheritedType {
                            span: target.span,
                            target: Path::class_like(self.arena, target.value),
                            provenance: Provenance::Direct,
                            arguments: &[],
                        });
                    }
                }
            }
        }

        if let Some(annotation) = annotation {
            for method in annotation.methods {
                methods.push(self.annotation_method(class_name, owner, method, origin));
            }
            for property in annotation.properties {
                properties.push(self.annotation_property(class_name, owner, property, origin));
            }
        }

        let constants = self.arena.alloc_slice_fill_iter(constants);
        let properties = self.arena.alloc_slice_fill_iter(properties);
        let methods = self.arena.alloc_slice_fill_iter(methods);
        let cases = self.arena.alloc_slice_fill_iter(cases);
        let uses = self.arena.alloc_slice_fill_iter(uses);

        OwnMembers {
            constants: ClassLikeConstantMemberList {
                index: sorted_offsets(self.arena, constants, |member| member.name.id),
                members: constants,
            },
            properties: PropertyMemberList {
                index: sorted_offsets(self.arena, properties, |member| member.name.id),
                members: properties,
                overrides: &[],
            },
            methods: MethodMemberList {
                index: sorted_offsets(self.arena, methods, |member| member.name.id),
                members: methods,
                overrides: &[],
            },
            cases: EnumCaseMemberList {
                index: sorted_offsets(self.arena, cases, |member| member.name.id),
                members: cases,
            },
            uses: InheritedTypeList { index: sorted_offsets(self.arena, uses, |edge| edge.target.id), edges: uses },
        }
    }

    fn method<I, St, Ex>(
        &mut self,
        class_name: &'arena [u8],
        owner: SymbolId,
        method: &Method<'arena, I, St, Ex>,
        origin: Origin,
    ) -> MethodMember<'arena> {
        let name = method.name.value;
        let mut flags = U32Flags::<crate::symbol::class_like::part::method::MethodFlag>::empty();
        use crate::symbol::class_like::part::method::MethodFlag;
        if has_modifier(method.modifiers, ModifierKind::Static) {
            flags = flags.with(MethodFlag::Static);
        }
        if has_modifier(method.modifiers, ModifierKind::Final) {
            flags = flags.with(MethodFlag::Final);
        }
        if has_modifier(method.modifiers, ModifierKind::Abstract) {
            flags = flags.with(MethodFlag::Abstract);
        }
        if method.flags.contains(HirMethodFlag::Yields) {
            flags = flags.with(MethodFlag::HasYield);
        }
        if method.flags.contains(HirMethodFlag::Throws) {
            flags = flags.with(MethodFlag::HasThrow);
        }
        if method.flags.contains(HirMethodFlag::ReturnsByReference) {
            flags = flags.with(MethodFlag::ReturnsByReference);
        }
        if method.flags.contains(HirMethodFlag::AssertionsInferred) {
            flags = flags.with(MethodFlag::AssertionsInferred);
        }
        if name.eq_ignore_ascii_case(b"__construct") {
            flags = flags.with(MethodFlag::Constructor);
        }
        if name.starts_with(b"__") {
            flags = flags.with(MethodFlag::Magic);
        }
        flags = flags.union(crate::linker::tags::method_flags(crate::linker::tags::tags_of(method.annotation)));

        let attributes = self.attributes(method.attributes);
        let generics = self.generics(SymbolId::method(class_name, name), method.annotation);
        let params = self.method_parameters(class_name, name, method.parameters.as_slice(), method.annotation, origin);
        let ret = self.type_slot_annotated(method.return_type, return_annotation(method.annotation));
        let where_constraints = self.where_constraints(method.annotation);
        let throws = self.throws(method.annotation);
        let assertions = self.assertions(method.annotation);
        let pure_unless_impure_params = self.pure_unless_impure_params(method.parameters.as_slice(), method.annotation);
        let self_out = self.self_out(method.annotation);
        let accessed_globals = self.accessed_globals(method.direct_accessed_globals);

        MethodMember {
            span: method.span,
            visibility: read_visibility(method.modifiers),
            name: Path::method(self.arena, class_name, name),
            defining_symbol: owner,
            flags,
            constraint: self.constraint(method.version_constraint),
            attributes,
            generics,
            params,
            ret,
            where_constraints,
            throws,
            assertions,
            pure_unless_impure_params,
            self_out,
            accessed_globals,
            origin,
        }
    }

    fn property<I, St, Ex>(
        &mut self,
        class_name: &'arena [u8],
        owner: SymbolId,
        property: &Property<'arena, I, St, Ex>,
        origin: Origin,
    ) -> PropertyMember<'arena> {
        let read = read_visibility(property.modifiers);
        let attributes = self.attributes(property.attributes);
        let ty = self.type_slot_annotated(property.r#type, var_annotation(property.annotation));

        PropertyMember {
            span: property.span,
            visibility: ReadWriteVisibility::new(read, write_visibility(property.modifiers, read)),
            name: Path::property(self.arena, class_name, property.variable.name),
            defining_symbol: owner,
            flags: property_flags(property.modifiers, property.default_value.is_some(), false)
                .union(crate::linker::tags::property_flags(crate::linker::tags::tags_of(property.annotation))),
            constraint: self.constraint(property.version_constraint),
            attributes,
            ty,
            hooks: &[],
            origin,
        }
    }

    fn hooked_property<I, St, Ex>(
        &mut self,
        class_name: &'arena [u8],
        owner: SymbolId,
        property: &HookedProperty<'arena, I, St, Ex>,
        origin: Origin,
    ) -> PropertyMember<'arena> {
        let property_name = property.variable.name;
        let property_id = SymbolId::property(class_name, property_name);

        let read = read_visibility(property.modifiers);
        let attributes = self.attributes(property.attributes);
        let ty = self.type_slot_annotated(property.r#type, var_annotation(property.annotation));

        let arena = self.arena;
        let hooks = arena.alloc_slice_fill_iter(
            property
                .hooks
                .as_slice()
                .iter()
                .map(|hook| self.property_hook(class_name, property_name, property_id, hook, ty, origin)),
        );

        PropertyMember {
            span: property.span,
            visibility: ReadWriteVisibility::new(read, write_visibility(property.modifiers, read)),
            name: Path::property(self.arena, class_name, property_name),
            defining_symbol: owner,
            flags: property_flags(property.modifiers, property.default_value.is_some(), !hooks.is_empty())
                .union(crate::linker::tags::property_flags(crate::linker::tags::tags_of(property.annotation))),
            constraint: self.constraint(property.version_constraint),
            attributes,
            ty,
            hooks,
            origin,
        }
    }

    fn property_hook<I, St, Ex>(
        &mut self,
        class_name: &'arena [u8],
        property_name: &'arena [u8],
        property_id: SymbolId,
        hook: &Hook<'arena, I, St, Ex>,
        property_ty: TypeSlot<'arena>,
        origin: Origin,
    ) -> PropertyHookMember<'arena> {
        let kind = if hook.name.value.eq_ignore_ascii_case(b"set") { HookKind::Set } else { HookKind::Get };
        let mut flags = U8Flags::<PropertyHookFlag>::empty();
        if has_modifier(hook.modifiers, ModifierKind::Final) {
            flags = flags.with(PropertyHookFlag::Final);
        }
        if has_modifier(hook.modifiers, ModifierKind::Abstract) {
            flags = flags.with(PropertyHookFlag::Abstract);
        }
        if hook.flags.contains(HookFlag::ReturnsByReference) {
            flags = flags.with(PropertyHookFlag::ReturnsByReference);
        }

        let attributes = self.attributes(hook.attributes);
        let parameter = hook
            .parameters
            .as_ref()
            .and_then(|parameters| parameters.as_slice().first())
            .map(|parameter| self.hook_parameter(class_name, property_name, hook.name.value, parameter, origin));

        PropertyHookMember {
            span: hook.span,
            kind,
            name: Path::property_hook(self.arena, class_name, property_name, hook.name.value),
            defining_symbol: property_id,
            flags,
            constraint: self.constraint(hook.version_constraint),
            attributes,
            parameter,
            ty: property_ty,
            origin,
        }
    }

    fn constant_member<I, St, Ex>(
        &mut self,
        class_name: &'arena [u8],
        owner: SymbolId,
        constant: &ClassLikeConstant<'arena, I, St, Ex>,
        origin: Origin,
    ) -> ClassLikeConstantMember<'arena> {
        let mut flags = U8Flags::<ClassLikeConstantFlag>::empty();
        if has_modifier(constant.modifiers, ModifierKind::Final) {
            flags = flags.with(ClassLikeConstantFlag::Final);
        }
        flags = flags.union(crate::linker::tags::constant_flags(crate::linker::tags::tags_of(constant.annotation)));

        let attributes = self.attributes(constant.attributes);
        let mut ty = self.type_slot(constant.r#type);
        ty.inferred = self.infer(constant.value);

        ClassLikeConstantMember {
            span: constant.span,
            visibility: read_visibility(constant.modifiers),
            name: Path::class_like_constant(self.arena, class_name, constant.name.value),
            defining_symbol: owner,
            flags,
            constraint: self.constraint(constant.version_constraint),
            attributes,
            ty,
            origin,
        }
    }

    fn enum_case<I, St, Ex>(
        &mut self,
        class_name: &'arena [u8],
        owner: SymbolId,
        case: &EnumCase<'arena, I, St, Ex>,
        origin: Origin,
    ) -> EnumCaseMember<'arena> {
        let mut flags = U8Flags::<EnumCaseFlag>::empty();
        if case.value.is_some() {
            flags = flags.with(EnumCaseFlag::Backed);
        } else {
            flags = flags.with(EnumCaseFlag::Unit);
        }

        let attributes = self.attributes(case.attributes);
        let value = case.value.and_then(|value| self.literal_atom(value));

        EnumCaseMember {
            span: case.span,
            name: Path::enum_case(self.arena, class_name, case.name.value),
            defining_symbol: owner,
            flags,
            constraint: self.constraint(case.version_constraint),
            attributes,
            value,
            origin,
        }
    }

    /// Lowers a literal expression (a backed enum case value) into its oracle
    /// atom. Returns `None` for non-literal constant expressions.
    fn literal_atom<I, St, Ex>(&mut self, expression: &Expression<'arena, I, St, Ex>) -> Option<Atom<'arena>> {
        let ExpressionKind::Literal(literal) = &expression.kind else {
            return None;
        };

        Some(match literal.kind {
            LiteralKind::Integer(integer) => Atom::int_literal(integer.value? as i64),
            LiteralKind::String(string) => self.builder.string_literal(string.value?),
            LiteralKind::Float(float) => Atom::float_literal(float.value.into_inner()),
            LiteralKind::True => crate::ty::well_known::TRUE,
            LiteralKind::False => crate::ty::well_known::FALSE,
            LiteralKind::Null => crate::ty::well_known::NULL,
        })
    }

    /// Synthesizes a virtual method declared by an `@method` docblock tag. The
    /// method has no body; its signature comes entirely from the annotation.
    fn annotation_method<I, St, Ex>(
        &mut self,
        class_name: &'arena [u8],
        owner: SymbolId,
        method: &MethodAnnotation<'arena, I, St, Ex>,
        origin: Origin,
    ) -> MethodMember<'arena> {
        let name = method.name.value;
        let mut flags = U32Flags::<MethodFlag>::empty();
        flags = flags.with(MethodFlag::Magic);
        if method.r#static {
            flags = flags.with(MethodFlag::Static);
        }

        let owner_method = SymbolId::method(class_name, name);
        let arena = self.arena;
        let parameters = arena.alloc_slice_fill_iter(method.parameters.as_slice().iter().map(|parameter| {
            let mut parameter_flags = U8Flags::<SignatureParameterFlag>::empty();
            if parameter.is_by_reference {
                parameter_flags = parameter_flags.with(SignatureParameterFlag::ByReference);
            }
            if parameter.is_variadic {
                parameter_flags = parameter_flags.with(SignatureParameterFlag::Variadic);
            }
            if parameter.default_value.is_some() {
                parameter_flags = parameter_flags.with(SignatureParameterFlag::HasDefault);
            }

            let mut ty = TypeSlot::new();
            ty.annotation = parameter.r#type.and_then(|r#type| self.lower_type_annotation(r#type));
            let default_ty = self.default_type_slot(parameter.default_value);

            SignatureParameter {
                span: parameter.span,
                defining_symbol: owner_method,
                path: Path::method_parameter(arena, class_name, name, parameter.variable.name),
                attributes: &[],
                flags: parameter_flags,
                constraint: crate::symbol::part::constraint::SymbolConstraint::unconstrained(),
                ty,
                out_ty: TypeSlot::new(),
                default_ty,
                origin,
            }
        }));

        let mut ret = TypeSlot::new();
        ret.annotation = method.return_type.and_then(|r#type| self.lower_type_annotation(r#type));
        let generics = self.generic_parameters(
            SymbolId::method(class_name, name),
            method.type_parameters.map_or(&[][..], |parameters| parameters.as_slice()),
        );

        MethodMember {
            span: method.span,
            visibility: annotation_visibility(method.visibility),
            name: Path::method(self.arena, class_name, name),
            defining_symbol: owner,
            flags,
            constraint: crate::symbol::part::constraint::SymbolConstraint::unconstrained(),
            attributes: &[],
            generics,
            params: parameters,
            ret,
            where_constraints: &[],
            throws: &[],
            assertions: &[],
            pure_unless_impure_params: &[],
            self_out: None,
            accessed_globals: &[],
            origin,
        }
    }

    /// Synthesizes a virtual property declared by an `@property` docblock tag.
    fn annotation_property(
        &mut self,
        class_name: &'arena [u8],
        owner: SymbolId,
        property: &PropertyAnnotation<'arena>,
        origin: Origin,
    ) -> PropertyMember<'arena> {
        let mut flags = U16Flags::<PropertyFlag>::empty();
        flags = flags.with(PropertyFlag::Magic).with(PropertyFlag::Virtual);
        if matches!(property.kind, PropertyAnnotationKind::Write) {
            flags = flags.with(PropertyFlag::Writeonly);
        }

        let mut ty = TypeSlot::new();
        ty.annotation = property.r#type.and_then(|r#type| self.lower_type_annotation(r#type));

        let write = match property.kind {
            PropertyAnnotationKind::Read => Visibility::Private,
            PropertyAnnotationKind::Write | PropertyAnnotationKind::ReadWrite => Visibility::Public,
        };

        PropertyMember {
            span: property.span,
            visibility: ReadWriteVisibility::new(Visibility::Public, write),
            name: Path::property(self.arena, class_name, property.variable.name),
            defining_symbol: owner,
            flags,
            constraint: crate::symbol::part::constraint::SymbolConstraint::unconstrained(),
            attributes: &[],
            ty,
            hooks: &[],
            origin,
        }
    }

    fn method_parameters<I, St, Ex>(
        &mut self,
        class_name: &'arena [u8],
        method_name: &'arena [u8],
        parameters: &[Parameter<'arena, I, St, Ex>],
        annotation: Option<&ItemAnnotation<'arena, I, St, Ex>>,
        origin: Origin,
    ) -> &'arena [SignatureParameter<'arena>] {
        let owner = SymbolId::method(class_name, method_name);
        let parameter_outs = annotation.map_or(&[][..], |annotation| annotation.parameter_outs);
        let arena = self.arena;
        arena.alloc_slice_fill_iter(parameters.iter().map(|parameter| {
            let flags = parameter_flags(parameter);
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
                path: Path::method_parameter(arena, class_name, method_name, parameter.variable.name),
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

    fn hook_parameter<I, St, Ex>(
        &mut self,
        class_name: &'arena [u8],
        property_name: &'arena [u8],
        hook_name: &'arena [u8],
        parameter: &Parameter<'arena, I, St, Ex>,
        origin: Origin,
    ) -> SignatureParameter<'arena> {
        let attributes = self.attributes(parameter.attributes);
        let ty = self
            .type_slot_annotated(parameter.r#type, parameter.annotation.map(|annotation| annotation.type_annotation));
        let default_ty = self.default_type_slot(parameter.default_value);

        SignatureParameter {
            span: parameter.span,
            defining_symbol: SymbolId::property_hook(class_name, property_name, hook_name),
            path: Path::property_hook_parameter(
                self.arena,
                class_name,
                property_name,
                hook_name,
                parameter.variable.name,
            ),
            attributes,
            flags: parameter_flags(parameter),
            constraint: self.constraint(parameter.version_constraint),
            ty,
            out_ty: TypeSlot::new(),
            default_ty,
            origin,
        }
    }
}

fn property_flags(
    modifiers: &[Modifier],
    has_default: bool,
    has_hooks: bool,
) -> U16Flags<crate::symbol::class_like::part::property::PropertyFlag> {
    use crate::symbol::class_like::part::property::PropertyFlag;
    let mut flags = U16Flags::<PropertyFlag>::empty();
    if has_modifier(modifiers, ModifierKind::Static) {
        flags = flags.with(PropertyFlag::Static);
    }
    if has_modifier(modifiers, ModifierKind::Readonly) {
        flags = flags.with(PropertyFlag::Readonly);
    }
    if has_modifier(modifiers, ModifierKind::Final) {
        flags = flags.with(PropertyFlag::Final);
    }
    if has_modifier(modifiers, ModifierKind::Abstract) {
        flags = flags.with(PropertyFlag::Abstract);
    }
    if has_default {
        flags = flags.with(PropertyFlag::HasDefault);
    }
    if has_hooks {
        flags = flags.with(PropertyFlag::Virtual);
    }

    flags
}

fn parameter_flags<I, St, Ex>(parameter: &Parameter<'_, I, St, Ex>) -> U8Flags<SignatureParameterFlag> {
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

    flags
}

pub(crate) fn has_modifier(modifiers: &[Modifier], kind: ModifierKind) -> bool {
    modifiers.iter().any(|modifier| modifier.kind == kind)
}

/// Maps an optional docblock visibility to the resolved visibility, defaulting
/// to public when the `@method` tag omits one.
fn annotation_visibility(visibility: Option<HirVisibility>) -> Visibility {
    match visibility.map(|visibility| visibility.kind) {
        Some(VisibilityKind::Private) => Visibility::Private,
        Some(VisibilityKind::Protected) => Visibility::Protected,
        _ => Visibility::Public,
    }
}

/// The `@var` type annotation declared in a property's docblock, if any.
fn var_annotation<'arena, I, St, Ex>(
    annotation: Option<&ItemAnnotation<'arena, I, St, Ex>>,
) -> Option<&'arena TypeAnnotation<'arena>> {
    annotation.and_then(|annotation| annotation.var.first()).map(|variable| variable.type_annotation)
}

fn read_visibility(modifiers: &[Modifier]) -> Visibility {
    if has_modifier(modifiers, ModifierKind::Private) {
        Visibility::Private
    } else if has_modifier(modifiers, ModifierKind::Protected) {
        Visibility::Protected
    } else {
        Visibility::Public
    }
}

fn write_visibility(modifiers: &[Modifier], read: Visibility) -> Visibility {
    if has_modifier(modifiers, ModifierKind::PrivateSet) {
        Visibility::Private
    } else if has_modifier(modifiers, ModifierKind::ProtectedSet) {
        Visibility::Protected
    } else if has_modifier(modifiers, ModifierKind::PublicSet) {
        Visibility::Public
    } else {
        read
    }
}
