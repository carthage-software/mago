mod r#enum;

use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;
use mago_hir::ir::item::member::MemberItem;
use mago_hir::ir::item::member::MemberItemKind;
use mago_hir::ir::item::member::constant::ClassLikeConstant;
use mago_hir::ir::item::member::enum_case::EnumCase;
use mago_hir::ir::item::member::method::Method;
use mago_hir::ir::item::modifier::Modifier;
use mago_hir::ir::item::modifier::ModifierKind;
use mago_hir::ir::item::statement::ItemStatement;
use mago_hir::ir::item::statement::ItemStatementKind;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::function_like::part::parameter::SignatureParameter;
use mago_oracle::ty::Type;
use mago_oracle::var::Var;
use mago_span::Span;

use crate::error::InferenceError;
use crate::error::InferenceResult;
use crate::flow::ControlFlow;
use crate::flow::Flow;
use crate::fold::Environment;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_statement_item(
        &mut self,
        span: Span,
        item: &'source ItemStatement<'source, SymbolId, S, E>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let kind = match item.kind {
            ItemStatementKind::Enum(node) => {
                ItemStatementKind::Enum(self.arena.alloc(self.infer_enum(item.meta, node)?))
            }
            ItemStatementKind::Class(_) => {
                return Err(InferenceError::Unsupported { span: item.span, construct: "class declarations" });
            }
            ItemStatementKind::Interface(_) => {
                return Err(InferenceError::Unsupported { span: item.span, construct: "interface declarations" });
            }
            ItemStatementKind::Trait(_) => {
                return Err(InferenceError::Unsupported { span: item.span, construct: "trait declarations" });
            }
            ItemStatementKind::Constant(_) => {
                return Err(InferenceError::Unsupported { span: item.span, construct: "constant declarations" });
            }
            ItemStatementKind::Function(_) => {
                return Err(InferenceError::Unsupported { span: item.span, construct: "function declarations" });
            }
        };

        let node = ItemStatement { meta: item.meta, span: item.span, kind };

        Ok(Statement {
            meta: Flow { reachable: self.reachable, exit: ControlFlow::Fallthrough },
            span,
            kind: StatementKind::Item(self.arena.alloc(node)),
        })
    }

    /// Folds one class-like member. `class_name` is the enclosing class's
    /// fully-qualified name and `this_type` the type a non-static method's `$this`
    /// takes. Properties are rejected (enums cannot declare them).
    pub(crate) fn infer_member(
        &mut self,
        class_name: &'arena [u8],
        this_type: Type<'arena>,
        member: &'source MemberItem<'source, SymbolId, S, E>,
    ) -> InferenceResult<MemberItem<'arena, SymbolId, Flow, Type<'arena>>> {
        let kind = match member.kind {
            MemberItemKind::Method(method) => {
                MemberItemKind::Method(self.arena.alloc(self.infer_method(class_name, this_type, method)?))
            }
            MemberItemKind::Constant(constant) => {
                MemberItemKind::Constant(self.arena.alloc(self.infer_constant_member(constant)?))
            }
            MemberItemKind::EnumCase(case) => MemberItemKind::EnumCase(self.arena.alloc(self.infer_enum_case(case)?)),
            MemberItemKind::TraitUse(_) => {
                return Err(InferenceError::Unsupported { span: member.span, construct: "trait use in an enum" });
            }
            MemberItemKind::Property(_) | MemberItemKind::HookedProperty(_) => {
                return Err(InferenceError::Unsupported { span: member.span, construct: "this member" });
            }
        };

        Ok(MemberItem { meta: member.meta, span: member.span, kind })
    }

    fn infer_method(
        &mut self,
        class_name: &'arena [u8],
        this_type: Type<'arena>,
        method: &'source Method<'source, SymbolId, S, E>,
    ) -> InferenceResult<Method<'arena, SymbolId, Flow, Type<'arena>>> {
        let attributes = self.infer_attributes(method.attributes)?;
        let annotation = match method.annotation {
            Some(annotation) => Some(&*self.arena.alloc(self.infer_item_annotation(annotation)?)),
            None => None,
        };
        let signature = self.method_signature(class_name, method.name.value);

        let outer_environment = std::mem::replace(&mut self.environment, Environment::new_in(self.source));
        let outer_reachable = self.reachable;
        self.reachable = true;

        if !is_static(method.modifiers) {
            self.environment.set(Var::new(self.arena.alloc_slice_copy(b"$this")), this_type);
        }
        self.bind_signature_parameters(method.parameters.items, signature);

        let parameters = self.infer_parameters(&method.parameters)?;
        let body = match method.body {
            Some(body) => Some(&*self.arena.alloc(self.infer_statement(body)?)),
            None => None,
        };

        self.environment = outer_environment;
        self.reachable = outer_reachable;

        Ok(Method {
            span: method.span,
            annotation,
            attributes,
            version_constraint: self.arena.alloc_slice_copy(method.version_constraint),
            flags: method.flags,
            modifiers: copy_slice_into(method.modifiers, self.arena),
            name: method.name.copy_into(self.arena),
            parameters,
            return_type: method.return_type.map(|return_type| copy_ref_into(return_type, self.arena)),
            direct_accessed_globals: copy_slice_into(method.direct_accessed_globals, self.arena),
            body,
        })
    }

    fn infer_constant_member(
        &mut self,
        constant: &'source ClassLikeConstant<'source, SymbolId, S, E>,
    ) -> InferenceResult<ClassLikeConstant<'arena, SymbolId, Flow, Type<'arena>>> {
        let attributes = self.infer_attributes(constant.attributes)?;
        let annotation = match constant.annotation {
            Some(annotation) => Some(&*self.arena.alloc(self.infer_item_annotation(annotation)?)),
            None => None,
        };
        let value = self.infer_expression(constant.value)?;

        Ok(ClassLikeConstant {
            span: constant.span,
            annotation,
            attributes,
            version_constraint: self.arena.alloc_slice_copy(constant.version_constraint),
            modifiers: copy_slice_into(constant.modifiers, self.arena),
            r#type: constant.r#type.map(|r#type| copy_ref_into(r#type, self.arena)),
            name: constant.name.copy_into(self.arena),
            value: self.arena.alloc(value),
            flattened: constant.flattened,
        })
    }

    fn infer_enum_case(
        &mut self,
        case: &'source EnumCase<'source, SymbolId, S, E>,
    ) -> InferenceResult<EnumCase<'arena, SymbolId, Flow, Type<'arena>>> {
        let attributes = self.infer_attributes(case.attributes)?;
        let annotation = match case.annotation {
            Some(annotation) => Some(&*self.arena.alloc(self.infer_item_annotation(annotation)?)),
            None => None,
        };
        let value = match case.value {
            Some(value) => Some(&*self.arena.alloc(self.infer_expression(value)?)),
            None => None,
        };

        Ok(EnumCase {
            span: case.span,
            annotation,
            attributes,
            version_constraint: self.arena.alloc_slice_copy(case.version_constraint),
            name: case.name.copy_into(self.arena),
            value,
        })
    }

    /// The signature parameters of `class_name::method_name`, or an empty slice
    /// when the class or method is unknown.
    fn method_signature(&self, class_name: &'arena [u8], method_name: &[u8]) -> &'arena [SignatureParameter<'arena>] {
        let target = SymbolId::method(class_name, method_name);
        self.symbols
            .get_class_like(SymbolId::class_like(class_name))
            .and_then(|symbol| symbol.methods().members.iter().find(|member| member.name.id == target))
            .map_or(&[], |member| member.params)
    }
}

fn is_static(modifiers: &[Modifier]) -> bool {
    modifiers.iter().any(|modifier| modifier.kind == ModifierKind::Static)
}
