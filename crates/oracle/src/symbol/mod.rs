use mago_allocator::prelude::*;
use mago_span::HasSpan;

use crate::id::SymbolId;
use crate::path::Path;
use crate::symbol::class_like::ClassLikeKind;
use crate::symbol::class_like::ClassLikeSymbol;
use crate::symbol::class_like::r#enum::EnumBackingType;
use crate::symbol::class_like::part::constant::ClassLikeConstantMember;
use crate::symbol::class_like::part::enum_case::EnumCaseMember;
use crate::symbol::class_like::part::inheritance::InheritedType;
use crate::symbol::class_like::part::property::PropertyMember;
use crate::symbol::constant::ConstantSymbol;
use crate::symbol::function_like::FunctionLikeSymbol;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::generic::GenericParameter;
use crate::symbol::part::origin::Origin;
use crate::ty::Type;

pub mod class_like;
pub mod constant;
pub mod function_like;
pub mod part;

/// The fully-resolved index of every symbol the analyzer knows about.
///
/// This is the central store the inference engine and checker query to answer
/// questions like "does `Foo` exist?", "what does `Foo::bar()` return?", or
/// "what classes descend from `Throwable`?". Entries are produced **already
/// linked** by a single linker pass - inheritance is flattened onto each
/// symbol, members carry their declaring symbol, and collisions between
/// declarations of the same name are resolved by [`Origin`] priority before
/// insertion. Consequently every query here is a hash lookup or a slice read,
/// never an inheritance walk.
///
/// Everything is keyed by [`SymbolId`] - the tagged, case-folded hash that
/// distinguishes a class `Foo` from a function `Foo` - so the three symbol
/// kinds live in separate maps without colliding. Class methods, properties,
/// constants, and enum cases are **not** top-level entries: they live inside
/// their [`ClassLikeSymbol`], reachable through its member lists.
///
/// The table holds only resolved definitions. Incremental-analysis state
/// (file signatures, safe-symbol sets) and diagnostics live in the engine
/// layer, not here.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug)]
pub struct SymbolTable<'arena, A: Arena> {
    /// Global constants, keyed by their [`SymbolId`].
    pub constants: HashMap<'arena, SymbolId, ConstantSymbol<'arena>, A>,
    /// Free functions, closures, and arrow functions, keyed by their
    /// [`SymbolId`]. Methods are *not* here - they belong to their class-like.
    pub function_likes: HashMap<'arena, SymbolId, FunctionLikeSymbol<'arena>, A>,
    /// Classes, interfaces, traits, enums, and anonymous classes, keyed by
    /// their [`SymbolId`].
    pub class_likes: HashMap<'arena, SymbolId, ClassLikeSymbol<'arena>, A>,
    /// Every namespace that contains at least one symbol, keyed by its
    /// case-folded [`SymbolId`] (PHP namespaces are case-insensitive). Lets the
    /// checker tell a missing class apart from a bare namespace reference.
    pub namespaces: HashSet<'arena, SymbolId, A>,
    /// The *direct* children of each class-like: classes that immediately
    /// extend it, interfaces that immediately extend it, or types that
    /// immediately implement/use it. The reverse of the inheritance edges
    /// stored on each symbol. Each slice is sorted by [`SymbolId`] for
    /// binary-search membership.
    pub direct_descendants: HashMap<'arena, SymbolId, &'arena [SymbolId], A>,
    /// The *transitive* closure of every descendant of each class-like,
    /// precomputed by the linker so subclass enumeration and sealed-type
    /// exhaustiveness are a single read rather than a repeated graph walk.
    /// Each slice is sorted by [`SymbolId`] for binary-search membership.
    pub all_descendants: HashMap<'arena, SymbolId, &'arena [SymbolId], A>,
}

impl<'arena, A: Arena> SymbolTable<'arena, A> {
    /// Creates an empty table whose maps allocate in `arena`.
    #[must_use]
    pub fn new_in(arena: &'arena A) -> Self {
        Self {
            constants: HashMap::new_in(arena),
            function_likes: HashMap::new_in(arena),
            class_likes: HashMap::new_in(arena),
            namespaces: HashSet::new_in(arena),
            direct_descendants: HashMap::new_in(arena),
            all_descendants: HashMap::new_in(arena),
        }
    }

    /// The global constant identified by `id`, or `None` if no such constant
    /// is known.
    #[inline]
    #[must_use]
    pub fn get_constant(&self, id: SymbolId) -> Option<&ConstantSymbol<'arena>> {
        self.constants.get(&id)
    }

    /// The free function, closure, or arrow function identified by `id`, or
    /// `None` if none is known. Methods are not function-likes - look them up
    /// through their [`ClassLikeSymbol`].
    #[inline]
    #[must_use]
    pub fn get_function_like(&self, id: SymbolId) -> Option<FunctionLikeSymbol<'arena>> {
        self.function_likes.get(&id).copied()
    }

    /// The class, interface, trait, enum, or anonymous class identified by
    /// `id`, or `None` if none is known.
    #[inline]
    #[must_use]
    pub fn get_class_like(&self, id: SymbolId) -> Option<ClassLikeSymbol<'arena>> {
        self.class_likes.get(&id).copied()
    }

    /// Whether `name` names a namespace that contains at least one symbol. The
    /// query is case-insensitive and tolerates a leading namespace separator;
    /// the name folds straight into a [`SymbolId`] with no allocation.
    #[inline]
    #[must_use]
    pub fn contains_namespace(&self, name: &[u8]) -> bool {
        self.namespaces.contains(&SymbolId::namespace(name))
    }

    /// Every descendant of `class_like`, transitively. Empty when the symbol
    /// is unknown or has no descendants.
    #[inline]
    #[must_use]
    pub fn descendants_of(&self, class_like: SymbolId) -> &'arena [SymbolId] {
        self.all_descendants.get(&class_like).copied().unwrap_or(&[])
    }

    /// The immediate children of `class_like`. Empty when the symbol is
    /// unknown or has no direct descendants.
    #[inline]
    #[must_use]
    pub fn direct_descendants_of(&self, class_like: SymbolId) -> &'arena [SymbolId] {
        self.direct_descendants.get(&class_like).copied().unwrap_or(&[])
    }

    /// Whether `candidate` transitively descends from `class_like` - a binary
    /// search over the precomputed, sorted descendant closure.
    #[inline]
    #[must_use]
    pub fn is_descendant(&self, class_like: SymbolId, candidate: SymbolId) -> bool {
        self.descendants_of(class_like).binary_search(&candidate).is_ok()
    }

    /// Whether `candidate` is an immediate child of `class_like` - a binary
    /// search over the sorted direct-descendant slice.
    #[inline]
    #[must_use]
    pub fn is_direct_descendant(&self, class_like: SymbolId, candidate: SymbolId) -> bool {
        self.direct_descendants_of(class_like).binary_search(&candidate).is_ok()
    }

    /// `true` iff `child` is the same class-like as `ancestor`, or extends /
    /// implements / uses-trait it transitively.
    #[must_use]
    pub fn descends_from(&self, child: SymbolId, ancestor: SymbolId) -> bool {
        child == ancestor || self.is_descendant(ancestor, child)
    }

    /// `true` iff `class` directly `use`s `trait_name` (the trait appears in
    /// `class`'s body as `use TraitName;`).
    ///
    /// Asymmetric vs [`descends_from`](Self::descends_from), which closes over
    /// inheritance: `descends_from` returns `true` for any trait in the chain,
    /// but `uses_trait` only for direct usage.
    #[must_use]
    pub fn uses_trait(&self, class: SymbolId, trait_name: SymbolId) -> bool {
        self.get_class_like(class).is_some_and(|symbol| symbol.uses().contains(trait_name))
    }

    /// How many type parameters `class` declares. `0` for unknown or
    /// non-generic classes.
    #[must_use]
    pub fn template_parameter_arity(&self, class: SymbolId) -> usize {
        self.get_class_like(class).map_or(0, |symbol| symbol.generics().len())
    }

    /// The type parameter at `position` in `class`'s declaration, or `None`
    /// if `position >= template_parameter_arity(class)`.
    #[must_use]
    pub fn template_parameter_at(&self, class: SymbolId, position: usize) -> Option<&'arena GenericParameter<'arena>> {
        self.get_class_like(class)?.generics().get(position)
    }

    /// The position of `class`'s type parameter named `name`, or `None` if
    /// no such parameter exists.
    #[must_use]
    pub fn template_parameter_index(&self, class: SymbolId, name: &[u8]) -> Option<usize> {
        self.get_class_like(class)?.generics().iter().position(|parameter| parameter.name == name)
    }

    /// The type `child` passes to `ancestor`'s `position`-th type parameter,
    /// expressed in `child`'s template namespace.
    ///
    /// For `class B<T> extends A<string>` with
    /// `inherited_template_argument(B, A, 0)`, returns `Some(string)`. For
    /// `class B<T> extends A<List<T>>`, returns `Some(List<T>)` - the caller
    /// substitutes `child`'s actual arguments to fully resolve.
    ///
    /// Returns `None` when `child` does not descend from `ancestor`, or when
    /// `position >= template_parameter_arity(ancestor)`.
    #[must_use]
    pub fn inherited_template_argument(
        &self,
        child: SymbolId,
        ancestor: SymbolId,
        position: usize,
    ) -> Option<Type<'arena>> {
        let child_symbol = self.get_class_like(child)?;
        let edge = child_symbol.inheritance_edge_to(ancestor)?;

        let ancestor_symbol = self.get_class_like(ancestor)?;
        let parameter = ancestor_symbol.generics().get(position)?;

        edge.arguments.iter().find(|argument| argument.parameter == parameter.name).map(|argument| argument.ty)
    }

    /// `true` iff the template parameter `(from_class, from_parameter)` is the
    /// *same variable* as `(to_class, to_parameter)` through inheritance
    /// forwarding: `from_class` (transitively) extends `to_class` binding its
    /// own `from_parameter` into `to_class`'s `to_parameter` slot
    /// (`class C<TC> extends D<TC>` forwards `C::TC` to `D::T`).
    ///
    /// This is the basis for inherited template subtyping: when it holds, a
    /// `from_parameter` value is also a `to_parameter` value (so `C::TC <:
    /// D::T`), one-directionally.
    ///
    /// The relation is transitively closed and reflexive.
    #[must_use]
    pub fn template_parameter_forwards_to(
        &self,
        from_class: SymbolId,
        from_parameter: &[u8],
        to_class: SymbolId,
        to_parameter: &[u8],
    ) -> bool {
        if from_class == to_class && from_parameter == to_parameter {
            return true;
        }

        let Some(symbol) = self.get_class_like(from_class) else {
            return false;
        };

        symbol.forwardings().iter().any(|forwarding| {
            forwarding.parameter == from_parameter
                && forwarding.target.defining_entity == to_class
                && forwarding.target.name == to_parameter
        })
    }

    /// `true` iff `class` declares or inherits a method named `method`.
    /// Mirrors PHP's `method_exists()` semantics: walks the inheritance
    /// closure (parent classes, implemented interfaces, used traits).
    #[must_use]
    pub fn class_has_method(&self, class: SymbolId, method: &[u8]) -> bool {
        let Some(symbol) = self.get_class_like(class) else {
            return false;
        };
        let target = SymbolId::method(class_name(&symbol), method);

        symbol.methods().members.iter().any(|member| member.name.id == target)
    }

    /// The declared type of `property` on `class`, walking the inheritance
    /// closure. `None` when the property is absent or its type is unknown.
    #[must_use]
    pub fn class_property_type(&self, class: SymbolId, property: &[u8]) -> Option<Type<'arena>> {
        let symbol = self.get_class_like(class)?;
        let target = SymbolId::property(class_name(&symbol), property);

        symbol
            .properties()?
            .members
            .iter()
            .find(|member| member.name.id == target)
            .and_then(|member| member.ty.effective(false))
    }

    /// `true` iff `class` declares or inherits a property named `property`.
    /// Mirrors PHP's `property_exists()` semantics.
    #[must_use]
    pub fn class_has_property(&self, class: SymbolId, property: &[u8]) -> bool {
        let Some(symbol) = self.get_class_like(class) else {
            return false;
        };
        let Some(properties) = symbol.properties() else {
            return false;
        };
        let target = SymbolId::property(class_name(&symbol), property);

        properties.members.iter().any(|member| member.name.id == target)
    }

    /// The backing of `enum_name`, or `None` for a pure enum, a non-enum, or an
    /// unknown name.
    #[must_use]
    pub fn enum_backing(&self, enum_name: SymbolId) -> Option<EnumBackingType> {
        match self.get_class_like(enum_name)? {
            ClassLikeSymbol::Enum(symbol) => symbol.backing_type,
            _ => None,
        }
    }

    /// What kind of class-like `name` declares (class, interface, enum, or
    /// trait), or `None` when the table doesn't know `name`.
    #[must_use]
    pub fn class_like_kind(&self, name: SymbolId) -> Option<ClassLikeKind> {
        Some(self.get_class_like(name)?.kind())
    }

    /// `true` iff `name` cannot be extended (PHP `final class` declaration,
    /// or any enum; enums are implicitly final).
    #[must_use]
    pub fn is_final(&self, name: SymbolId) -> bool {
        match self.get_class_like(name) {
            Some(ClassLikeSymbol::Class(class)) => class.is_final(),
            Some(ClassLikeSymbol::Enum(_) | ClassLikeSymbol::AnonymousClass(_)) => true,
            _ => false,
        }
    }

    /// The recorded body of `class::alias` (a `@type` alias declared on the
    /// class), or `None` when the alias is unknown.
    #[must_use]
    pub fn alias_body(&self, class: SymbolId, alias: &[u8]) -> Option<Type<'arena>> {
        let symbol = self.get_class_like(class)?;
        let target = SymbolId::type_alias(class_name(&symbol), alias);

        symbol
            .aliases()
            .members
            .iter()
            .find(|member| member.name.id == target)
            .and_then(|member| member.ty.effective(true))
    }

    /// The declared or inferred type of `class::constant`. `None` when the
    /// constant is unknown.
    #[must_use]
    pub fn class_constant_type(&self, class: SymbolId, constant: &[u8]) -> Option<Type<'arena>> {
        let symbol = self.get_class_like(class)?;
        let target = SymbolId::class_like_constant(class_name(&symbol), constant);

        symbol
            .constants()
            .members
            .iter()
            .find(|member| member.name.id == target)
            .and_then(|member| member.ty.effective(true))
    }

    /// Every constant visible from `class`'s scope (its own and those it
    /// inherits), as a borrowed slice. Empty when `class` is unknown or
    /// declares no constants.
    #[must_use]
    pub fn class_constants(&self, class: SymbolId) -> &[ClassLikeConstantMember<'arena>] {
        self.get_class_like(class).map_or(&[], |symbol| symbol.constants().members)
    }

    /// Every case `enum_name` declares, as a borrowed slice. Empty for
    /// non-enums or unknown names.
    #[must_use]
    pub fn enum_cases(&self, enum_name: SymbolId) -> &[EnumCaseMember<'arena>] {
        match self.get_class_like(enum_name) {
            Some(ClassLikeSymbol::Enum(symbol)) => symbol.cases.members,
            _ => &[],
        }
    }

    /// The declared or inferred type of a global constant. `None` when the
    /// name is unknown.
    #[must_use]
    pub fn global_constant_type(&self, name: SymbolId) -> Option<Type<'arena>> {
        self.get_constant(name).and_then(|constant| constant.ty.effective(true))
    }

    /// How many properties `class` declares or inherits (visible from
    /// `class`'s scope).
    #[must_use]
    pub fn class_property_count(&self, class: SymbolId) -> usize {
        self.get_class_like(class)
            .and_then(|symbol| symbol.properties())
            .map_or(0, |properties| properties.members.len())
    }

    /// The `position`-th property of `class`, walking the inheritance closure
    /// in declaration order.
    #[must_use]
    pub fn class_property_at(&self, class: SymbolId, position: usize) -> Option<&'arena PropertyMember<'arena>> {
        self.get_class_like(class)?.properties()?.members.get(position)
    }

    /// The closed list of *direct* inheritors of `class_like` when it is
    /// sealed; `None` when it is open (anything may extend it).
    #[must_use]
    pub fn sealed_direct_inheritors(&self, class_like: SymbolId) -> Option<&[InheritedType<'arena>]> {
        if let Some(permitted) = self.get_class_like(class_like)?.permitted_inheritors()
            && !permitted.is_empty()
        {
            Some(permitted)
        } else {
            None
        }
    }

    /// If `child` is a direct inheritor of a sealed class, returns that sealed
    /// parent. `None` otherwise.
    #[must_use]
    pub fn sealed_parent_of(&self, child: SymbolId) -> Option<Path<'arena>> {
        self.get_class_like(child)?
            .sealed_parents()
            .first()
            .and_then(|&parent| self.get_class_like(parent))
            .map(|parent| parent.path())
    }
}

fn class_name<'arena>(symbol: &ClassLikeSymbol<'arena>) -> &'arena [u8] {
    symbol.path().segments.first().map_or(&[][..], |segment| segment.as_bytes())
}

/// A top-level, resolved symbol: a class-like, a function-like, or a global
/// constant.
///
/// "Top-level" is what separates a `Symbol` from a [`SymbolMember`]: a symbol
/// stands on its own and is keyed directly in the [`SymbolTable`], whereas a
/// member (method, property, ...) only exists inside another symbol. Every symbol
/// carries an identity ([`Path`]/[`SymbolId`]), a provenance ([`Origin`]), the
/// PHP-version range it is available in, and the attributes applied to its
/// declaration.
pub trait Symbol<'arena>: Sized + HasSpan {
    /// The symbol's identity hash. Defaults to the id of its [`Path`].
    fn id(&self) -> SymbolId {
        self.path().id
    }

    /// The symbol's fully-qualified path - its identity plus the displayable
    /// segments of its name.
    fn path(&self) -> Path<'arena>;

    /// Where the symbol comes from (project, dependency, runtime, or an
    /// override stub), which decides who wins a same-name collision.
    fn origin(&self) -> Origin;

    /// Whether the symbol is a polyfill: a declaration standing in for one the
    /// runtime would otherwise provide.
    fn is_polyfill(&self) -> bool;

    /// The attributes written on the symbol's declaration.
    fn applied_attributes(&self) -> &'arena [AppliedAttribute<'arena>];

    /// The constraint gating which PHP versions this symbol is available in,
    /// or `None` when its availability is not constrained.
    fn constraint(&self) -> Option<SymbolConstraint<'arena>>;
}

/// A member that lives inside another symbol: a method, property, property hook,
/// class constant, or enum case.
///
/// A member is not a standalone [`Symbol`] - it has no entry of its own in the
/// [`SymbolTable`] and is reached through its owning class-like. Beyond a
/// symbol's identity and provenance, a member knows the
/// [`defining_symbol`](Self::defining_symbol) it belongs to, which for an
/// inherited member is the ancestor that actually declares it rather than the
/// class the member was reached through.
pub trait SymbolMember<'arena> {
    /// The member's identity hash. Defaults to the id of its [`Path`].
    fn id(&self) -> SymbolId {
        self.path().id
    }

    /// The member's fully-qualified path (e.g. `Foo::$bar`).
    fn path(&self) -> Path<'arena>;

    /// The class-like that declares this member - the ancestor, for an
    /// inherited member, not the class it was reached through.
    fn defining_symbol(&self) -> SymbolId;

    /// Where the member comes from (project, dependency, runtime, or override).
    fn origin(&self) -> Origin;

    /// The attributes written on the member's declaration.
    fn applied_attributes(&self) -> &'arena [AppliedAttribute<'arena>];

    /// The constraint gating which PHP versions this member is available in,
    /// or `None` when its availability is not constrained.
    fn constraint(&self) -> Option<SymbolConstraint<'arena>>;
}
