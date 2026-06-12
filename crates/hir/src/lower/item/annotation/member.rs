use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_phpdoc_syntax::cst;
use mago_phpdoc_syntax::cst::tag::MethodTagValue;
use mago_phpdoc_syntax::cst::tag::PropertyTagValue;
use mago_span::HasSpan;

use crate::ir::delimited::Delimited;
use crate::ir::identifier::Identifier;
use crate::ir::item::annotation::generics::TypeParameterDefiningEntity;
use crate::ir::item::annotation::generics::Variance;
use crate::ir::item::annotation::member::MethodAnnotation;
use crate::ir::item::annotation::member::PropertyAnnotation;
use crate::ir::item::annotation::member::PropertyAnnotationKind;
use crate::ir::item::modifier::Visibility;
use crate::ir::item::modifier::VisibilityKind;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_method_annotation(
        &mut self,
        method: &'scratch MethodTagValue<'scratch>,
        owner: Identifier<'arena>,
    ) -> MethodAnnotation<'arena, (), (), ()> {
        let name = self.phpdoc_name(&method.name);
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::Method(owner, name));

        let type_parameters = method.templates.map(|templates| {
            let mut entries = Vec::new_in(self.arena);
            for entry in templates.entries.iter() {
                let annotation =
                    self.lower_type_parameter_annotation(&entry.template, Variance::from(entry.template.variance));
                self.type_resolution.add_template(annotation.name, annotation.bound, annotation.default);
                entries.push(annotation);
            }

            Delimited { span: templates.less_than.join(templates.greater_than), items: entries.leak() }
        });

        let parameters = Delimited {
            span: method.parameters.left_parenthesis.join(method.parameters.right_parenthesis),
            items: self.arena.alloc_slice_fill_iter(
                method.parameters.entries.iter().map(|parameter| self.lower_parameter_annotation(parameter)),
            ),
        };
        let return_type = method.return_type.map(|return_type| self.lower_type_annotation(return_type));

        self.type_resolution.leave_scope();

        MethodAnnotation {
            span: method.span(),
            r#static: method.r#static.is_some(),
            visibility: match &method.visibility {
                Some(v) => match v {
                    cst::Visibility::Public(k) => Some(Visibility { span: k.span, kind: VisibilityKind::Public }),
                    cst::Visibility::Protected(k) => Some(Visibility { span: k.span, kind: VisibilityKind::Protected }),
                    cst::Visibility::Private(k) => Some(Visibility { span: k.span, kind: VisibilityKind::Private }),
                },
                None => None,
            },
            name,
            type_parameters,
            parameters,
            return_type,
        }
    }

    pub(crate) fn lower_property_annotation(
        &mut self,
        property: &'scratch PropertyTagValue<'scratch>,
        kind: PropertyAnnotationKind,
    ) -> PropertyAnnotation<'arena> {
        PropertyAnnotation {
            span: property.span(),
            kind,
            r#type: property.r#type.map(|r#type| self.lower_type_annotation(r#type)),
            variable: self.phpdoc_variable(&property.variable),
        }
    }
}
