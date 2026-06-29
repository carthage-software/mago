use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::vec::Vec;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::item::parameter::Parameter;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::function_like::part::parameter::SignatureParameter;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_oracle::var::Var;

use crate::error::InferenceError;
use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    /// Binds each parameter into the environment under its declared signature
    /// type (the native hint or `@param`), defaulting to `mixed`.
    pub(crate) fn bind_signature_parameters(
        &mut self,
        parameters: &[Parameter<'source, SymbolId, S, E>],
        signature: &[SignatureParameter<'arena>],
    ) {
        for (index, parameter) in parameters.iter().enumerate() {
            let ty = signature.get(index).and_then(|parameter| parameter.ty.effective(true)).unwrap_or(TYPE_MIXED);
            let name = Var::new(self.arena.alloc_slice_copy(parameter.variable.name));
            self.environment.set(name, ty);
        }
    }

    /// Infers each parameter node: its attributes and default value are inferred,
    /// the remaining (type-only) fields are carried through.
    pub(crate) fn infer_parameters(
        &mut self,
        parameters: &Delimited<'source, Parameter<'source, SymbolId, S, E>>,
    ) -> InferenceResult<Delimited<'arena, Parameter<'arena, SymbolId, Flow, Type<'arena>>>> {
        let mut items = Vec::new_in(self.arena);
        for parameter in parameters.items {
            items.push(self.infer_parameter(parameter)?);
        }

        Ok(Delimited { span: parameters.span, items: items.leak() })
    }

    fn infer_parameter(
        &mut self,
        parameter: &Parameter<'source, SymbolId, S, E>,
    ) -> InferenceResult<Parameter<'arena, SymbolId, Flow, Type<'arena>>> {
        let attributes = self.infer_attributes(parameter.attributes)?;
        let default_value = match parameter.default_value {
            Some(default_value) => Some(&*self.arena.alloc(self.infer_expression(default_value)?)),
            None => None,
        };

        let hooks = match parameter.hooks {
            None => None,
            Some(_) => {
                return Err(InferenceError::Unsupported {
                    span: parameter.span,
                    construct: "parameter property hooks",
                });
            }
        };

        Ok(Parameter {
            span: parameter.span,
            annotation: parameter.annotation.map(|annotation| copy_ref_into(annotation, self.arena)),
            attributes,
            flags: parameter.flags,
            version_constraint: self.arena.alloc_slice_copy(parameter.version_constraint),
            modifiers: self.arena.alloc_slice_copy(parameter.modifiers),
            r#type: parameter.r#type.map(|r#type| copy_ref_into(r#type, self.arena)),
            variable: parameter.variable.copy_into(self.arena),
            default_value,
            hooks,
        })
    }
}
