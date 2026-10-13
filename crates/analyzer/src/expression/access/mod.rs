use std::rc::Rc;

use mago_allocator::Arena;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::union::TUnion;
use mago_syntax::cst::Access;
use mago_syntax::cst::Call;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Literal;
use mago_syntax::cst::Variable;
use mago_word::word;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::utils::expression::get_block_expression_id;

pub mod class_constant_access;
pub mod property_access;
pub mod static_property_access;

mod cached_receiver;

/// Captures skipped receivers from saved types without analyzing them again.
pub(super) fn record_cached_receiver<'ctx, 'arena, A>(
    context: &mut Context<'ctx, 'arena, A>,
    block_context: &BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    expression: &Expression<'arena>,
) -> Result<(), AnalysisError>
where
    A: Arena,
{
    if artifacts.editor.is_none() {
        return Ok(());
    }
    match expression.unparenthesized() {
        Expression::Variable(Variable::Direct(variable)) => {
            if let Some(ty) = block_context.locals.get(&word(variable.name)).cloned() {
                artifacts.set_rc_expression_type(variable, ty);
            }

            Ok(())
        }
        Expression::Access(Access::ClassConstant(access)) => {
            record_cached_receiver(context, block_context, artifacts, access.class)?;
            if let Some((reference, _)) = cached_receiver::constant(context, block_context, access)
                && let Some(editor) = &mut artifacts.editor
            {
                editor.members.insert(reference);
            }

            Ok(())
        }
        Expression::Call(Call::Method(call)) => {
            record_cached_receiver(context, block_context, artifacts, call.object)?;
            let references = cached_receiver::methods(context, block_context, artifacts, call);
            if let Some(editor) = &mut artifacts.editor {
                editor.methods.extend(references);
            }

            Ok(())
        }
        Expression::Access(Access::Property(access)) => property_access::add_memoized_property_reference(
            context,
            block_context,
            artifacts,
            access.object,
            &access.property,
            false,
        ),
        Expression::Access(Access::NullSafeProperty(access)) => property_access::add_memoized_property_reference(
            context,
            block_context,
            artifacts,
            access.object,
            &access.property,
            true,
        ),
        Expression::Access(Access::StaticProperty(access)) => {
            static_property_access::add_memoized_static_property_reference(
                context,
                block_context,
                artifacts,
                access.class,
                &access.property,
            )
        }
        Expression::ArrayAccess(access) => {
            record_cached_receiver(context, block_context, artifacts, access.array)?;
            record_cached_receiver(context, block_context, artifacts, access.index)
        }
        _ => Ok(()),
    }
}

/// Reads saved types and literal array offsets without running expression analysis.
pub(super) fn saved_receiver_type<'ctx, 'arena, A>(
    context: &Context<'ctx, 'arena, A>,
    block_context: &BlockContext<'ctx>,
    artifacts: &AnalysisArtifacts,
    expression: &Expression<'arena>,
) -> Option<Rc<TUnion>>
where
    A: Arena,
{
    artifacts.editor.as_ref()?;
    if let Some(saved) = artifacts.get_rc_expression_type(expression).cloned().or_else(|| {
        get_block_expression_id(expression, context, block_context)
            .and_then(|id| block_context.locals.get(&id).cloned())
    }) {
        return Some(saved);
    }

    match expression.unparenthesized() {
        Expression::Access(Access::ClassConstant(access)) => {
            return cached_receiver::constant(context, block_context, access).map(|(_, ty)| Rc::new(ty));
        }
        Expression::Call(Call::Method(call)) => {
            return cached_receiver::method_return(context, block_context, artifacts, call).map(Rc::new);
        }
        _ => {}
    }

    let Expression::ArrayAccess(access) = expression.unparenthesized() else {
        return None;
    };

    let key = match access.index.unparenthesized() {
        Expression::Literal(Literal::Integer(integer)) => ArrayKey::Integer(i64::try_from(integer.value?).ok()?),
        Expression::Literal(Literal::String(string)) => ArrayKey::from_string(word(string.value?)),
        _ => saved_receiver_type(context, block_context, artifacts, access.index)?.get_single_array_key()?,
    };

    let container = saved_receiver_type(context, block_context, artifacts, access.array)?;
    let mut value = None;
    for atomic in container.types.iter() {
        let TAtomic::Array(array) = atomic else {
            return None;
        };

        let known = match array {
            TArray::Keyed(array) => array.known_items.as_ref().and_then(|items| items.get(&key)),
            TArray::List(array) => {
                let index = usize::try_from(key.get_integer()?).ok()?;
                array.known_elements.as_ref().and_then(|items| items.get(&index))
            }
        };

        let element = known.map(|(_, element)| element.clone()).or_else(|| array.get_value_type())?;
        value = Some(add_optional_union_type(element, value.as_ref(), context.codebase));
    }

    value.map(Rc::new)
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Access<'arena> {
    fn analyze<'ctx, A>(
        &'ast self,
        context: &mut Context<'ctx, 'arena, A>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>
    where
        A: Arena,
    {
        match self {
            Access::Property(access) => access.analyze(context, block_context, artifacts),
            Access::NullSafeProperty(access) => access.analyze(context, block_context, artifacts),
            Access::StaticProperty(access) => access.analyze(context, block_context, artifacts),
            Access::ClassConstant(access) => access.analyze(context, block_context, artifacts),
        }
    }
}
