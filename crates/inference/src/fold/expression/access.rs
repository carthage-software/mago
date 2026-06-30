use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::vec::Vec;
use mago_hir::ir::expression::Access;
use mago_hir::ir::expression::AccessKind;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::selector::ConstantSelector;
use mago_hir::ir::expression::selector::ConstantSelectorKind;
use mago_hir::ir::expression::selector::MemberSelector;
use mago_hir::ir::expression::selector::MemberSelectorKind;
use mago_hir::ir::variable::Variable;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::Symbol;
use mago_oracle::symbol::class_like::ClassLikeKind;
use mago_oracle::symbol::class_like::ClassLikeSymbol;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::array::ArrayAtom;
use mago_oracle::ty::atom::payload::array::ArrayKey;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_oracle::ty::well_known::TYPE_NULL;
use mago_span::Span;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;
use crate::reconciler::meet_with;
use crate::semantics::collect_closed_array;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_access(
        &mut self,
        span: Span,
        access: &'source Access<'source, SymbolId, S, E>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        match &access.kind {
            AccessKind::Array(array, index) => {
                let array = self.infer_expression(array)?;
                let index = self.infer_expression(index)?;
                let shape = self.array_element_type(array.meta, index.meta);
                let meta = match self.array_place_id(&array, index.meta) {
                    Some(place) => self.environment.lookup(place).unwrap_or(shape),
                    None => shape,
                };

                let node = Access {
                    span: access.span,
                    kind: AccessKind::Array(self.arena.alloc(array), self.arena.alloc(index)),
                };

                Ok(Expression { meta, span, kind: ExpressionKind::Access(self.arena.alloc(node)) })
            }
            AccessKind::StaticProperty(class, property) => {
                self.infer_static_property(span, access.span, class, property)
            }
            AccessKind::ClassConstant(class, selector) => self.infer_class_constant(span, access.span, class, selector),
            AccessKind::Property(object, selector) => self.infer_property(span, access.span, object, selector, false),
            AccessKind::NullsafeProperty(object, selector) => {
                self.infer_property(span, access.span, object, selector, true)
            }
        }
    }

    fn infer_property(
        &mut self,
        span: Span,
        access_span: Span,
        object: &'source Expression<'source, SymbolId, S, E>,
        selector: &'source MemberSelector<'source, SymbolId, S, E>,
        nullsafe: bool,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let object = self.infer_expression(object)?;

        let mut meta = match &selector.kind {
            MemberSelectorKind::Name(name) => {
                let declared = self.property_read_type(object.meta, name.value);
                match self.property_place_id(&object, name.value) {
                    Some(place) => self.environment.lookup(place).unwrap_or(declared),
                    None => declared,
                }
            }
            _ => TYPE_MIXED,
        };

        if nullsafe && object.meta.atoms.iter().any(|atom| matches!(atom, Atom::Null)) {
            meta = self.union(meta, TYPE_NULL);
        }

        let object = self.arena.alloc(object);
        let selector = self.infer_member_selector(selector)?;
        let kind = if nullsafe {
            AccessKind::NullsafeProperty(object, selector)
        } else {
            AccessKind::Property(object, selector)
        };

        let node = Access { span: access_span, kind };
        Ok(Expression { meta, span, kind: ExpressionKind::Access(self.arena.alloc(node)) })
    }

    fn infer_static_property(
        &mut self,
        span: Span,
        access_span: Span,
        class: &'source Expression<'source, SymbolId, S, E>,
        property: &'source Variable<'source, SymbolId, S, E>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let meta = self.static_property_type(class, property);
        let class = self.infer_expression(class)?;
        let property = self.infer_variable_node(property)?;

        let node = Access { span: access_span, kind: AccessKind::StaticProperty(self.arena.alloc(class), property) };

        Ok(Expression { meta, span, kind: ExpressionKind::Access(self.arena.alloc(node)) })
    }

    fn infer_class_constant(
        &mut self,
        span: Span,
        access_span: Span,
        class: &'source Expression<'source, SymbolId, S, E>,
        selector: &'source ConstantSelector<'source, SymbolId, S, E>,
    ) -> InferenceResult<Expression<'arena, SymbolId, Flow, Type<'arena>>> {
        let meta = self.class_constant_type(class, selector);
        let class = self.infer_expression(class)?;
        let selector = self.infer_constant_selector(selector)?;

        let node = Access { span: access_span, kind: AccessKind::ClassConstant(self.arena.alloc(class), selector) };

        Ok(Expression { meta, span, kind: ExpressionKind::Access(self.arena.alloc(node)) })
    }

    /// The declared type of a `Class::$property` static-property read. `mixed`
    /// when the class, the property, or its type is unknown, or when a property of
    /// that name exists but is not static.
    fn static_property_type(
        &self,
        class: &'source Expression<'source, SymbolId, S, E>,
        property: &Variable<'source, SymbolId, S, E>,
    ) -> Type<'arena> {
        let Some(symbol) = self.resolve_class(class) else {
            return TYPE_MIXED;
        };
        let Variable::Direct(direct) = property else {
            return TYPE_MIXED;
        };

        self.declared_property_type(&symbol, direct.name, true).unwrap_or(TYPE_MIXED)
    }

    fn property_read_type(&self, receiver: Type<'arena>, name: &[u8]) -> Type<'arena> {
        let mut found = None;
        for atom in receiver.atoms {
            let candidate = match atom {
                Atom::Object(object) => Some(object.name.as_bytes()),
                Atom::Enum(_) => return TYPE_MIXED,
                _ => continue,
            };

            if found.is_some() {
                return TYPE_MIXED;
            }
            found = candidate;
        }

        let Some(class_name) = found else {
            return TYPE_MIXED;
        };
        let Some(symbol) = self.symbols.get_class_like(SymbolId::class_like(class_name)) else {
            return TYPE_MIXED;
        };

        self.declared_property_type(&symbol, name, false).unwrap_or(TYPE_MIXED)
    }

    fn declared_property_type(
        &self,
        class: &ClassLikeSymbol<'arena>,
        name: &[u8],
        want_static: bool,
    ) -> Option<Type<'arena>> {
        let target = SymbolId::property(class.path().as_bytes(), name);
        class
            .properties()?
            .members
            .iter()
            .find(|member| member.name.id == target && member.is_static() == want_static)
            .and_then(|member| member.ty.effective(false))
    }

    /// The type of a `Class::CONSTANT` read. Handles three forms: `Class::class`
    /// yields a literal class-string, an enum case (`Status::Active`) yields that
    /// case's singleton type, and anything else resolves the class constant.
    /// `mixed` when the class or member is unknown, or the selector is dynamic.
    fn class_constant_type(
        &mut self,
        class: &'source Expression<'source, SymbolId, S, E>,
        selector: &ConstantSelector<'source, SymbolId, S, E>,
    ) -> Type<'arena> {
        let ConstantSelectorKind::Name(name) = selector.kind else {
            return TYPE_MIXED;
        };
        let Some(symbol) = self.resolve_class(class) else {
            return TYPE_MIXED;
        };

        let class_name = symbol.path().as_bytes();
        let class_id = symbol.path().id;

        if matches!(name.value, b"class") {
            let atom = self.ty.class_string_literal(class_name);
            return self.ty.union_of(&[atom]);
        }

        if matches!(symbol.kind(), ClassLikeKind::Enum) {
            let case = SymbolId::enum_case(class_name, name.value);
            if self.symbols.enum_cases(class_id).iter().any(|member| member.name.id == case) {
                let atom = self.ty.enum_case(class_name, name.value);
                return self.ty.union_of(&[atom]);
            }
        }

        self.symbols.class_constant_type(class_id, name.value).unwrap_or(TYPE_MIXED)
    }

    fn infer_constant_selector(
        &mut self,
        selector: &ConstantSelector<'source, SymbolId, S, E>,
    ) -> InferenceResult<ConstantSelector<'arena, SymbolId, Flow, Type<'arena>>> {
        let kind = match selector.kind {
            ConstantSelectorKind::Missing => ConstantSelectorKind::Missing,
            ConstantSelectorKind::Name(name) => ConstantSelectorKind::Name(name.copy_into(self.arena)),
            ConstantSelectorKind::Expression(expression) => {
                let expression = self.infer_expression(expression)?;

                ConstantSelectorKind::Expression(self.arena.alloc(expression))
            }
        };

        Ok(ConstantSelector { span: selector.span, kind })
    }

    /// Resolves the class operand of a static access to its symbol, trying the
    /// written name then its short form (mirroring function resolution). `None`
    /// for `self`/`static`/`parent` (no class context here) or an unknown class.
    pub(crate) fn resolve_class(
        &self,
        class: &'source Expression<'source, SymbolId, S, E>,
    ) -> Option<ClassLikeSymbol<'arena>> {
        let identifier = match &class.kind {
            ExpressionKind::Identifier(identifier) | ExpressionKind::Constant(identifier) => identifier,
            ExpressionKind::Self_ | ExpressionKind::Static => {
                return self.self_class.and_then(|name| self.symbols.get_class_like(SymbolId::class_like(name)));
            }
            ExpressionKind::Parent => return self.resolve_parent(),
            _ => return None,
        };

        if let Some(symbol) = self.symbols.get_class_like(SymbolId::class_like(identifier.value)) {
            return Some(symbol);
        }

        if identifier.is_local() && !identifier.imported {
            let short = identifier.last_segment();
            if short != identifier.value {
                return self.symbols.get_class_like(SymbolId::class_like(short));
            }
        }

        None
    }

    /// The direct superclass of the current `self::` context, for `parent::`.
    /// `None` outside a class or when the class has no parent.
    fn resolve_parent(&self) -> Option<ClassLikeSymbol<'arena>> {
        let class = self.self_class?;
        let symbol = self.symbols.get_class_like(SymbolId::class_like(class))?;
        let (parent, _) = symbol.inheritance_edges();

        self.symbols.get_class_like(SymbolId::class_like(parent?.target.as_bytes()))
    }

    pub(crate) fn array_element_type(&mut self, array: Type<'arena>, key: Type<'arena>) -> Type<'arena> {
        if let [Atom::Array(atom)] = array.atoms {
            return self.keyed_element_type(atom, key);
        }

        let mut items = Vec::new_in(self.source);
        if !collect_closed_array(array, &mut items) {
            return TYPE_MIXED;
        }

        let Some(key) = self.array_key_of(key) else {
            let mut atoms = Vec::new_in(self.source);
            for item in &items {
                atoms.extend_from_slice(item.value.atoms);
            }

            atoms.push(Atom::Null);

            return self.ty.union_of(&atoms);
        };

        match items.iter().find(|item| item.key == key) {
            Some(item) if item.optional => self.union(item.value, TYPE_NULL),
            Some(item) => item.value,
            None => TYPE_NULL,
        }
    }

    fn keyed_element_type(&mut self, atom: &ArrayAtom<'arena>, key: Type<'arena>) -> Type<'arena> {
        let literal_key = self.array_key_of(key);

        let mut atoms = Vec::new_in(self.source);
        let mut definitely_present = false;

        if let Some(known_items) = atom.known_items {
            for item in known_items {
                let item_key = self.array_key_type(item.key);
                if !meet_with(&mut self.ty, self.symbols, key, item_key).is_never() {
                    atoms.extend_from_slice(item.value.atoms);
                }

                if !item.optional && literal_key == Some(item.key) {
                    definitely_present = true;
                }
            }
        }

        if let (Some(rest_key), Some(rest_value)) = (atom.key_param, atom.value_param)
            && !meet_with(&mut self.ty, self.symbols, key, rest_key).is_never()
        {
            atoms.extend_from_slice(rest_value.atoms);
        }

        if !definitely_present {
            atoms.push(Atom::Null);
        }

        self.ty.union_of(&atoms)
    }

    fn array_key_type(&mut self, key: ArrayKey<'arena>) -> Type<'arena> {
        match key {
            ArrayKey::Int(value) => self.int_literal(value),
            ArrayKey::String(value) => self.literal_string(value),
            ArrayKey::Const { .. } => TYPE_MIXED,
        }
    }

    pub(crate) fn remove_array_key(&mut self, array: Type<'arena>, key: ArrayKey<'arena>) -> Type<'arena> {
        let mut items = Vec::new_in(self.source);
        if !collect_closed_array(array, &mut items) {
            return array;
        }

        let mut kept = Vec::new_in(self.source);
        for item in &items {
            if item.key != key {
                kept.push(*item);
            }
        }

        self.closed_array(&kept)
    }
}
