use mago_allocator::Arena;
use mago_flags::U8Flags;
use mago_flags::U16Flags;
use mago_hir::ir::identifier::Identifier;
use mago_hir::ir::item::annotation::ItemAnnotation;
use mago_hir::ir::item::modifier::ModifierKind;
use mago_hir::ir::item::statement::r#enum::EnumBackingTypeKind;
use mago_hir::ir::r#type::annotation::TypeAnnotationKind;
use mago_span::Span;

use crate::linker::ClassLikeRef;
use crate::linker::index::sorted_offsets;
use crate::linker::lower::Lowerer;
use crate::linker::members::has_modifier;
use crate::path::Path;
use crate::symbol::class_like::ClassLikeSymbol;
use crate::symbol::class_like::anonymous_class::AnonymousClassSymbol;
use crate::symbol::class_like::class::ClassFlag;
use crate::symbol::class_like::class::ClassSymbol;
use crate::symbol::class_like::r#enum::EnumBackingType;
use crate::symbol::class_like::r#enum::EnumSymbol;
use crate::symbol::class_like::interface::InterfaceSymbol;
use crate::symbol::class_like::part::alias::TypeAliasMemberList;
use crate::symbol::class_like::part::inheritance::InheritedType;
use crate::symbol::class_like::part::inheritance::InheritedTypeList;
use crate::symbol::class_like::part::inheritance::Provenance;
use crate::symbol::class_like::r#trait::TraitSymbol;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::origin::Origin;

impl<'arena, S, A> Lowerer<'_, '_, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    /// Lowers the `@type` aliases and `@import-type` imports declared on a
    /// class-like into a member list. A local alias carries its bridged body; an
    /// imported alias carries a reference to the source class's alias.
    fn type_aliases<I, St, Ex>(
        &mut self,
        class_name: &'arena [u8],
        annotation: Option<&ItemAnnotation<'arena, I, St, Ex>>,
        origin: Origin,
    ) -> TypeAliasMemberList<'arena> {
        use crate::symbol::class_like::part::alias::TypeAliasMember;
        use crate::symbol::part::ty::TypeSlot;

        let Some(annotation) = annotation else {
            return TypeAliasMemberList { members: &[], index: &[] };
        };
        if annotation.type_aliases.is_empty() && annotation.imported_type_aliases.is_empty() {
            return TypeAliasMemberList { members: &[], index: &[] };
        }

        let defining_symbol = crate::id::SymbolId::class_like(class_name);
        let mut members = self.builder.scratch_vec();

        for alias in annotation.type_aliases {
            let mut ty = TypeSlot::new();
            ty.annotation = self.lower_type_annotation(alias.r#type);
            members.push(TypeAliasMember {
                span: alias.span,
                name: Path::type_alias(self.arena, class_name, alias.name.value),
                defining_symbol,
                ty,
                origin,
            });
        }

        for imported in annotation.imported_type_aliases {
            let source = self.builder.intern_class_like_path(imported.from.value);
            let source_alias = self.builder.intern(imported.name.value);
            let reference = self
                .builder
                .alias(crate::ty::atom::payload::alias::AliasAtom { class_name: source, alias_name: source_alias });
            let mut ty = TypeSlot::new();
            ty.annotation = Some(self.builder.union_of(&[reference]));

            let local_name = imported.r#as.unwrap_or(imported.name).value;
            members.push(TypeAliasMember {
                span: imported.span,
                name: Path::type_alias(self.arena, class_name, local_name),
                defining_symbol,
                ty,
                origin,
            });
        }

        let arena = self.arena;
        let members = arena.alloc_slice_fill_iter(members);

        TypeAliasMemberList { index: sorted_offsets(arena, members, |member| member.name.id), members }
    }

    /// Builds an [`InheritedTypeList`] from direct edges to `targets`, each a
    /// class-like name written in the source (no generic arguments yet).
    fn direct_inheritance(&self, targets: &[Identifier<'arena>]) -> InheritedTypeList<'arena> {
        let arena = self.arena;
        let edges = arena.alloc_slice_fill_iter(targets.iter().map(|target| InheritedType {
            span: target.span,
            target: Path::class_like(arena, target.value),
            provenance: Provenance::Direct,
            arguments: &[],
        }));
        let index = sorted_offsets(arena, edges, |edge| edge.target.id);

        InheritedTypeList { edges, index }
    }

    /// Builds an [`InheritedTypeList`] from `(span, name)` pairs.
    fn inheritance_list(&self, edges: impl Iterator<Item = (Span, &'arena [u8])>) -> InheritedTypeList<'arena> {
        let arena = self.arena;
        let edges = arena.alloc_slice_fill_iter(edges.map(|(span, name)| InheritedType {
            span,
            target: Path::class_like(arena, name),
            provenance: Provenance::Direct,
            arguments: &[],
        }));

        let index = sorted_offsets(arena, edges, |edge| edge.target.id);

        InheritedTypeList { edges, index }
    }

    /// Lowers the `@mixin` declarations of a class-like into an inheritance list;
    /// only mixins naming a class-like contribute an edge.
    fn mixins<I, St, Ex>(&self, annotation: Option<&ItemAnnotation<'arena, I, St, Ex>>) -> InheritedTypeList<'arena> {
        let Some(annotation) = annotation else {
            return InheritedTypeList { edges: &[], index: &[] };
        };

        self.inheritance_list(annotation.mixins.iter().filter_map(|mixin| match &mixin.r#type {
            TypeAnnotationKind::Named(named) => Some((mixin.span, crate::linker::types::reference_name(&named.kind))),
            _ => None,
        }))
    }

    /// Lowers the `@require-extends` declarations of a trait or interface.
    fn require_extends<I, St, Ex>(
        &self,
        annotation: Option<&ItemAnnotation<'arena, I, St, Ex>>,
    ) -> InheritedTypeList<'arena> {
        let Some(annotation) = annotation else {
            return InheritedTypeList { edges: &[], index: &[] };
        };

        self.inheritance_list(
            annotation
                .require_extends
                .iter()
                .map(|require| (require.span, crate::linker::types::reference_name(&require.r#type.kind))),
        )
    }

    /// Lowers the `@require-implements` declarations of a trait or interface.
    fn require_implements<I, St, Ex>(
        &self,
        annotation: Option<&ItemAnnotation<'arena, I, St, Ex>>,
    ) -> InheritedTypeList<'arena> {
        let Some(annotation) = annotation else {
            return InheritedTypeList { edges: &[], index: &[] };
        };

        self.inheritance_list(
            annotation
                .require_implements
                .iter()
                .map(|require| (require.span, crate::linker::types::reference_name(&require.r#type.kind))),
        )
    }

    /// Lowers a class-like winner into its declared symbol shell: identity,
    /// flags, and direct inheritance edges. Member lists and the flattened
    /// inheritance closure are filled by later phases.
    pub(crate) fn class_like_shell<St, Ex>(
        &mut self,
        reference: &ClassLikeRef<'_, 'arena, St, Ex>,
    ) -> ClassLikeSymbol<'arena> {
        match reference {
            ClassLikeRef::Class(class, origin) => {
                let members = self.own_members(class.name.value, class.members.as_slice(), class.annotation, *origin);
                let extends = class.extends.and_then(|extends| extends.types.first()).map(|target| InheritedType {
                    span: target.span,
                    target: Path::class_like(self.arena, target.value),
                    provenance: Provenance::Direct,
                    arguments: &[],
                });
                let implements =
                    self.direct_inheritance(class.implements.map_or(&[][..], |implements| implements.types));
                let generics = self.generics(crate::id::SymbolId::class_like(class.name.value), class.annotation);
                let aliases = self.type_aliases(class.name.value, class.annotation, *origin);

                let mut flags = U16Flags::<ClassFlag>::empty();
                if has_modifier(class.modifiers, ModifierKind::Final) {
                    flags = flags.with(ClassFlag::Final);
                }
                if has_modifier(class.modifiers, ModifierKind::Abstract) {
                    flags = flags.with(ClassFlag::Abstract);
                }
                if has_modifier(class.modifiers, ModifierKind::Readonly) {
                    flags = flags.with(ClassFlag::Readonly);
                }
                flags = flags.union(crate::linker::tags::class_flags(crate::linker::tags::tags_of(class.annotation)));

                let symbol = ClassSymbol {
                    span: class.span,
                    origin: *origin,
                    name: Path::class_like(self.arena, class.name.value),
                    flags,
                    constraint: self.constraint(class.version_constraint),
                    attributes: self.attributes(class.attributes),
                    generics,
                    forwardings: &[],
                    aliases,
                    extends,
                    implements,
                    uses: members.uses,
                    mixins: self.mixins(class.annotation),
                    permitted_inheritors: &[],
                    sealed_parents: &[],
                    constants: members.constants,
                    properties: members.properties,
                    methods: members.methods,
                };

                ClassLikeSymbol::Class(self.arena.alloc(symbol))
            }
            ClassLikeRef::Interface(interface, origin) => {
                let members =
                    self.own_members(interface.name.value, interface.members.as_slice(), interface.annotation, *origin);
                let extends = self.direct_inheritance(interface.extends.map_or(&[][..], |extends| extends.types));
                let generics =
                    self.generics(crate::id::SymbolId::class_like(interface.name.value), interface.annotation);
                let aliases = self.type_aliases(interface.name.value, interface.annotation, *origin);

                let symbol = InterfaceSymbol {
                    span: interface.span,
                    origin: *origin,
                    name: Path::class_like(self.arena, interface.name.value),
                    flags: crate::linker::tags::interface_flags(crate::linker::tags::tags_of(interface.annotation)),
                    constraint: self.constraint(interface.version_constraint),
                    attributes: self.attributes(interface.attributes),
                    generics,
                    forwardings: &[],
                    aliases,
                    extends,
                    require_extends: self.require_extends(interface.annotation),
                    require_implements: self.require_implements(interface.annotation),
                    mixins: self.mixins(interface.annotation),
                    permitted_inheritors: &[],
                    sealed_parents: &[],
                    constants: members.constants,
                    properties: members.properties,
                    methods: members.methods,
                };

                ClassLikeSymbol::Interface(self.arena.alloc(symbol))
            }
            ClassLikeRef::Trait(r#trait, origin) => {
                let members =
                    self.own_members(r#trait.name.value, r#trait.members.as_slice(), r#trait.annotation, *origin);
                let generics = self.generics(crate::id::SymbolId::class_like(r#trait.name.value), r#trait.annotation);
                let aliases = self.type_aliases(r#trait.name.value, r#trait.annotation, *origin);

                let symbol = TraitSymbol {
                    span: r#trait.span,
                    origin: *origin,
                    name: Path::class_like(self.arena, r#trait.name.value),
                    flags: crate::linker::tags::trait_flags(crate::linker::tags::tags_of(r#trait.annotation)),
                    constraint: self.constraint(r#trait.version_constraint),
                    attributes: self.attributes(r#trait.attributes),
                    generics,
                    forwardings: &[],
                    aliases,
                    uses: members.uses,
                    require_extends: self.require_extends(r#trait.annotation),
                    require_implements: self.require_implements(r#trait.annotation),
                    mixins: self.mixins(r#trait.annotation),
                    constants: members.constants,
                    properties: members.properties,
                    methods: members.methods,
                };

                ClassLikeSymbol::Trait(self.arena.alloc(symbol))
            }
            ClassLikeRef::Enum(r#enum, origin) => {
                let members =
                    self.own_members(r#enum.name.value, r#enum.members.as_slice(), r#enum.annotation, *origin);
                let backing_type = r#enum.backing_type.as_ref().and_then(|backing| match backing.kind {
                    EnumBackingTypeKind::Int => Some(EnumBackingType::Int),
                    EnumBackingTypeKind::String => Some(EnumBackingType::String),
                    EnumBackingTypeKind::Invalid(_) => None,
                });
                // PHP enums implicitly implement `UnitEnum` (providing `cases()`),
                // and backed enums additionally implement `BackedEnum` (providing
                // `from()` / `tryFrom()`). These are not written in the source, so
                // they are synthesized here as inheritance edges.
                let span = r#enum.span;
                let unit_enum: &'arena [u8] = b"UnitEnum";
                let backed_enum: &'arena [u8] = b"BackedEnum";
                let implicit =
                    std::iter::once((span, unit_enum)).chain(backing_type.is_some().then_some((span, backed_enum)));
                let natives = r#enum
                    .implements
                    .map_or(&[][..], |implements| implements.types)
                    .iter()
                    .map(|target| (target.span, target.value));
                let implements = self.inheritance_list(natives.chain(implicit));
                let aliases = self.type_aliases(r#enum.name.value, r#enum.annotation, *origin);

                let symbol = EnumSymbol {
                    span: r#enum.span,
                    origin: *origin,
                    name: Path::class_like(self.arena, r#enum.name.value),
                    flags: crate::linker::tags::enum_flags(crate::linker::tags::tags_of(r#enum.annotation)),
                    backing_type,
                    constraint: self.constraint(r#enum.version_constraint),
                    attributes: self.attributes(r#enum.attributes),
                    aliases,
                    implements,
                    uses: members.uses,
                    sealed_parents: &[],
                    constants: members.constants,
                    cases: members.cases,
                    methods: members.methods,
                };

                ClassLikeSymbol::Enum(self.arena.alloc(symbol))
            }
            ClassLikeRef::AnonymousClass(anonymous_class, origin) => {
                let members = self.own_members(
                    anonymous_class.name,
                    anonymous_class.members.as_slice(),
                    anonymous_class.annotation,
                    *origin,
                );
                let extends =
                    anonymous_class.extends.and_then(|extends| extends.types.first()).map(|target| InheritedType {
                        span: target.span,
                        target: Path::class_like(self.arena, target.value),
                        provenance: Provenance::Direct,
                        arguments: &[],
                    });
                let implements =
                    self.direct_inheritance(anonymous_class.implements.map_or(&[][..], |implements| implements.types));

                let symbol = AnonymousClassSymbol {
                    span: anonymous_class.span,
                    origin: *origin,
                    name: Path::class_like(self.arena, anonymous_class.name),
                    flags: U8Flags::empty(),
                    constraint: SymbolConstraint::unconstrained(),
                    attributes: self.attributes(anonymous_class.attributes),
                    extends,
                    implements,
                    uses: members.uses,
                    mixins: self.mixins(anonymous_class.annotation),
                    sealed_parents: &[],
                    constants: members.constants,
                    properties: members.properties,
                    methods: members.methods,
                };

                ClassLikeSymbol::AnonymousClass(self.arena.alloc(symbol))
            }
        }
    }
}

/// The fully-qualified name a class-like reference declares, used to key it and
/// to seed namespace tracking.
pub(crate) fn class_like_name<'arena, St, Ex>(reference: &ClassLikeRef<'_, 'arena, St, Ex>) -> &'arena [u8] {
    match reference {
        ClassLikeRef::Class(class, _) => class.name.value,
        ClassLikeRef::Interface(interface, _) => interface.name.value,
        ClassLikeRef::Trait(r#trait, _) => r#trait.name.value,
        ClassLikeRef::Enum(r#enum, _) => r#enum.name.value,
        ClassLikeRef::AnonymousClass(anonymous_class, _) => anonymous_class.name,
    }
}
