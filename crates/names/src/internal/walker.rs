use mago_allocator::prelude::*;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::Attribute;
use mago_syntax::cst::Binary;
use mago_syntax::cst::BinaryOperator;
use mago_syntax::cst::Class;
use mago_syntax::cst::ClassConstantAccess;
use mago_syntax::cst::Constant;
use mago_syntax::cst::ConstantAccess;
use mago_syntax::cst::Enum;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Extends;
use mago_syntax::cst::Function;
use mago_syntax::cst::FunctionCall;
use mago_syntax::cst::FunctionPartialApplication;
use mago_syntax::cst::Hint;
use mago_syntax::cst::Identifier;
use mago_syntax::cst::Implements;
use mago_syntax::cst::Instantiation;
use mago_syntax::cst::Interface;
use mago_syntax::cst::Namespace;
use mago_syntax::cst::StaticMethodCall;
use mago_syntax::cst::StaticMethodPartialApplication;
use mago_syntax::cst::StaticPropertyAccess;
use mago_syntax::cst::Trait;
use mago_syntax::cst::TraitUse;
use mago_syntax::cst::Use;
use mago_syntax::cst::UseItem;
use mago_syntax::cst::UseItems;
use mago_syntax::cst::UseType;
use mago_syntax::walker::MutWalker;

use crate::NameResolutionMetadata;
use crate::ResolvedNames;
use crate::internal::context::NameResolutionContext;
use crate::kind::NameKind;
use crate::scope::concat_with_sep;
use crate::scope::trim_start_byte;

/// An AST visitor (`MutWalker`) that traverses a PHP Abstract Syntax Tree
/// to resolve names (classes, functions, constants, etc.) according to
/// PHP's scoping and aliasing rules.
#[derive(Debug, Clone, Default)]
pub struct NameWalker<'arena> {
    /// Accumulates the resolved names found during the AST walk.
    pub resolved_names: ResolvedNames<'arena>,
    /// Start offset of the current namespace declaration, or `None` for the
    /// file's top-level global namespace.
    import_scope: Option<u32>,
}

impl<'arena> NameWalker<'arena> {
    fn insert(
        &mut self,
        span: Span,
        name: &'arena [u8],
        imported: bool,
        kind: NameKind,
        original: &'arena [u8],
        root: &'arena [u8],
    ) {
        let metadata = NameResolutionMetadata::new(kind, original, root, self.import_scope);
        self.resolved_names.insert_with_metadata(span, name, imported, metadata);
    }

    fn insert_identifier(
        &mut self,
        identifier: Identifier<'arena>,
        name: &'arena [u8],
        imported: bool,
        kind: NameKind,
    ) {
        let original = identifier.value();
        self.insert(identifier.span(), name, imported, kind, original, source_root(original));
    }
}

fn source_root(name: &[u8]) -> &[u8] {
    let name = name.strip_prefix(b"\\").unwrap_or(name);
    let root_end = name.iter().position(|byte| *byte == b'\\').unwrap_or(name.len());
    &name[..root_end]
}

fn use_root<'arena>(item: &UseItem<'arena>) -> &'arena [u8] {
    item.alias.as_ref().map_or_else(|| item.name.last_segment(), |alias| alias.identifier.value)
}

impl<'ast, 'arena, A> MutWalker<'ast, 'arena, NameResolutionContext<'arena, A>> for NameWalker<'arena>
where
    A: Arena,
{
    fn walk_in_namespace(
        &mut self,
        namespace: &'ast Namespace<'arena>,
        context: &mut NameResolutionContext<'arena, A>,
    ) {
        context.exit_namespace();
        self.import_scope = Some(namespace.namespace.span.start.offset);

        if let Some(ns) = namespace.name.as_ref() {
            self.resolved_names.insert_at(ns.span(), ns.value(), false);
        }

        context.enter_namespace(namespace.name.as_ref().map(mago_syntax::cst::Identifier::value));
    }

    fn walk_in_use(&mut self, r#use: &'ast Use<'arena>, context: &mut NameResolutionContext<'arena, A>) {
        context.populate_from_use(r#use);

        match &r#use.items {
            UseItems::Sequence(seq) => {
                for item in &seq.items {
                    let fqn = trim_start_byte(item.name.value(), b'\\');
                    self.insert(item.name.span(), fqn, true, NameKind::Default, item.name.value(), use_root(item));
                }
            }
            UseItems::TypedSequence(seq) => {
                let kind = match seq.r#type {
                    UseType::Function(_) => NameKind::Function,
                    UseType::Const(_) => NameKind::Constant,
                };

                for item in &seq.items {
                    let fqn = trim_start_byte(item.name.value(), b'\\');
                    self.insert(item.name.span(), fqn, true, kind, item.name.value(), use_root(item));
                }
            }
            UseItems::TypedList(list) => {
                let kind = match list.r#type {
                    UseType::Function(_) => NameKind::Function,
                    UseType::Const(_) => NameKind::Constant,
                };
                let prefix = trim_start_byte(list.namespace.value(), b'\\');
                self.resolved_names.insert_at(list.namespace.span(), context.intern(prefix), true);
                for item in &list.items {
                    let fqn = context.intern(&concat_with_sep(&[prefix, item.name.value()], b'\\'));
                    self.insert(item.name.span(), fqn, true, kind, item.name.value(), use_root(item));
                }
            }
            UseItems::MixedList(list) => {
                let prefix = trim_start_byte(list.namespace.value(), b'\\');
                self.resolved_names.insert_at(list.namespace.span(), context.intern(prefix), true);
                for mixed in &list.items {
                    let kind = match mixed.r#type {
                        Some(UseType::Function(_)) => NameKind::Function,
                        Some(UseType::Const(_)) => NameKind::Constant,
                        None => NameKind::Default,
                    };
                    let fqn = context.intern(&concat_with_sep(&[prefix, mixed.item.name.value()], b'\\'));
                    self.insert(
                        mixed.item.name.span(),
                        fqn,
                        true,
                        kind,
                        mixed.item.name.value(),
                        use_root(&mixed.item),
                    );
                }
            }
        }
    }

    fn walk_in_constant(&mut self, constant: &'ast Constant<'arena>, context: &mut NameResolutionContext<'arena, A>) {
        for item in &constant.items {
            let name = context.qualify_name(item.name.value);

            self.insert(item.name.span, name, false, NameKind::Constant, item.name.value, item.name.value);
        }
    }

    fn walk_in_function(&mut self, function: &'ast Function<'arena>, context: &mut NameResolutionContext<'arena, A>) {
        let name = context.qualify_name(function.name.value);

        self.insert(function.name.span, name, false, NameKind::Function, function.name.value, function.name.value);
    }

    fn walk_in_class(&mut self, class: &'ast Class<'arena>, context: &mut NameResolutionContext<'arena, A>) {
        let classlike = context.qualify_name(class.name.value);

        self.insert(class.name.span, classlike, false, NameKind::Default, class.name.value, class.name.value);
    }

    fn walk_in_interface(
        &mut self,
        interface: &'ast Interface<'arena>,
        context: &mut NameResolutionContext<'arena, A>,
    ) {
        let classlike = context.qualify_name(interface.name.value);

        self.insert(
            interface.name.span,
            classlike,
            false,
            NameKind::Default,
            interface.name.value,
            interface.name.value,
        );
    }

    fn walk_in_trait(&mut self, r#trait: &'ast Trait<'arena>, context: &mut NameResolutionContext<'arena, A>) {
        let classlike = context.qualify_name(r#trait.name.value);

        self.insert(r#trait.name.span, classlike, false, NameKind::Default, r#trait.name.value, r#trait.name.value);
    }

    fn walk_in_enum(&mut self, r#enum: &'ast Enum<'arena>, context: &mut NameResolutionContext<'arena, A>) {
        let classlike = context.qualify_name(r#enum.name.value);

        self.insert(r#enum.name.span, classlike, false, NameKind::Default, r#enum.name.value, r#enum.name.value);
    }

    fn walk_in_trait_use(&mut self, trait_use: &'ast TraitUse<'arena>, context: &mut NameResolutionContext<'arena, A>) {
        for trait_name in &trait_use.trait_names {
            let (trait_classlike, imported) = context.resolve(NameKind::Default, trait_name.value());

            self.insert_identifier(*trait_name, trait_classlike, imported, NameKind::Default);
        }
    }

    fn walk_in_extends(&mut self, extends: &'ast Extends<'arena>, context: &mut NameResolutionContext<'arena, A>) {
        for parent in &extends.types {
            let (parent_classlike, imported) = context.resolve(NameKind::Default, parent.value());

            self.insert_identifier(*parent, parent_classlike, imported, NameKind::Default);
        }
    }

    fn walk_in_implements(
        &mut self,
        implements: &'ast Implements<'arena>,
        context: &mut NameResolutionContext<'arena, A>,
    ) {
        for parent in &implements.types {
            let (parent_classlike, imported) = context.resolve(NameKind::Default, parent.value());

            self.insert_identifier(*parent, parent_classlike, imported, NameKind::Default);
        }
    }

    fn walk_in_hint(&mut self, hint: &'ast Hint<'arena>, context: &mut NameResolutionContext<'arena, A>) {
        if let Hint::Identifier(identifier) = hint {
            let (name, imported) = context.resolve(NameKind::Default, identifier.value());

            self.insert_identifier(*identifier, name, imported, NameKind::Default);
        }
    }

    fn walk_in_attribute(
        &mut self,
        attribute: &'ast Attribute<'arena>,
        context: &mut NameResolutionContext<'arena, A>,
    ) {
        let (name, imported) = context.resolve(NameKind::Default, attribute.name.value());

        self.insert_identifier(attribute.name, name, imported, NameKind::Default);
    }

    fn walk_in_function_call(
        &mut self,
        function_call: &'ast FunctionCall<'arena>,
        context: &mut NameResolutionContext<'arena, A>,
    ) {
        if let Expression::Identifier(identifier) = function_call.function {
            let (name, imported) = context.resolve(NameKind::Function, identifier.value());

            self.insert_identifier(*identifier, name, imported, NameKind::Function);
        }
    }

    fn walk_in_function_partial_application(
        &mut self,
        function_partial_application: &'ast FunctionPartialApplication<'arena>,
        context: &mut NameResolutionContext<'arena, A>,
    ) {
        if let Expression::Identifier(identifier) = function_partial_application.function {
            let (name, imported) = context.resolve(NameKind::Function, identifier.value());

            self.insert_identifier(*identifier, name, imported, NameKind::Function);
        }
    }

    fn walk_in_instantiation(
        &mut self,
        instantiation: &'ast Instantiation<'arena>,
        context: &mut NameResolutionContext<'arena, A>,
    ) {
        if let Expression::Identifier(identifier) = instantiation.class {
            let (name, imported) = context.resolve(NameKind::Default, identifier.value());

            self.insert_identifier(*identifier, name, imported, NameKind::Default);
        }
    }

    fn walk_in_static_method_call(
        &mut self,
        static_method_call: &'ast StaticMethodCall<'arena>,
        context: &mut NameResolutionContext<'arena, A>,
    ) {
        if let Expression::Identifier(identifier) = static_method_call.class {
            let (name, imported) = context.resolve(NameKind::Default, identifier.value());

            self.insert_identifier(*identifier, name, imported, NameKind::Default);
        }
    }

    fn walk_in_static_method_partial_application(
        &mut self,
        static_method_partial_application: &'ast StaticMethodPartialApplication<'arena>,
        context: &mut NameResolutionContext<'arena, A>,
    ) {
        if let Expression::Identifier(identifier) = static_method_partial_application.class {
            let (name, imported) = context.resolve(NameKind::Default, identifier.value());

            self.insert_identifier(*identifier, name, imported, NameKind::Default);
        }
    }

    fn walk_in_static_property_access(
        &mut self,
        static_property_access: &'ast StaticPropertyAccess<'arena>,
        context: &mut NameResolutionContext<'arena, A>,
    ) {
        if let Expression::Identifier(identifier) = static_property_access.class {
            let (name, imported) = context.resolve(NameKind::Default, identifier.value());

            self.insert_identifier(*identifier, name, imported, NameKind::Default);
        }
    }

    fn walk_in_class_constant_access(
        &mut self,
        class_constant_access: &'ast ClassConstantAccess<'arena>,
        context: &mut NameResolutionContext<'arena, A>,
    ) {
        if let Expression::Identifier(identifier) = class_constant_access.class {
            let (name, imported) = context.resolve(NameKind::Default, identifier.value());

            self.insert_identifier(*identifier, name, imported, NameKind::Default);
        }
    }

    fn walk_in_binary(&mut self, binary: &'ast Binary<'arena>, context: &mut NameResolutionContext<'arena, A>) {
        if let (BinaryOperator::Instanceof(_), Expression::Identifier(identifier)) = (binary.operator, binary.rhs) {
            let (name, imported) = context.resolve(NameKind::Default, identifier.value());

            self.insert_identifier(*identifier, name, imported, NameKind::Default);
        }
    }

    fn walk_in_constant_access(
        &mut self,
        constant_access: &'ast ConstantAccess<'arena>,
        context: &mut NameResolutionContext<'arena, A>,
    ) {
        let identifier = &constant_access.name;

        if !self.resolved_names.contains(&identifier.span().start) {
            let (name, imported) = context.resolve(NameKind::Constant, identifier.value());

            self.insert_identifier(*identifier, name, imported, NameKind::Constant);
        }
    }

    fn walk_out_namespace(&mut self, _namespace: &Namespace<'arena>, context: &mut NameResolutionContext<'arena, A>) {
        context.exit_namespace();
        self.import_scope = None;
    }
}
