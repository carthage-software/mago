use mago_phpdoc_syntax::cst::tag::MethodTagValue;
use mago_phpdoc_syntax::cst::tag::PropertyTagValue;
use mago_phpdoc_syntax::cst::tag::TypeAliasImportTagValue;
use mago_phpdoc_syntax::cst::tag::TypeAliasTagValue;
use mago_span::HasSpan;

use crate::ir::generics::Variance;
use crate::ir::member::annotation::ImportedTypeAliasAnnotation;
use crate::ir::member::annotation::MethodAnnotation;
use crate::ir::member::annotation::PropertyAnnotation;
use crate::ir::member::annotation::PropertyAnnotationKind;
use crate::ir::member::annotation::TypeAliasAnnotation;
use crate::ir::variable::DirectVariable;
use crate::lower::Lowering;

impl<'arena> Lowering<'arena> {
    pub(crate) fn lower_type_alias_annotation(
        &self,
        alias: &'arena TypeAliasTagValue<'arena>,
    ) -> TypeAliasAnnotation<'arena> {
        TypeAliasAnnotation {
            span: alias.span(),
            name: self.phpdoc_name(&alias.alias),
            r#type: self.lower_type_annotation(alias.r#type),
        }
    }

    pub(crate) fn lower_imported_type_alias_annotation(
        &self,
        import: &'arena TypeAliasImportTagValue<'arena>,
    ) -> ImportedTypeAliasAnnotation<'arena> {
        ImportedTypeAliasAnnotation {
            span: import.span(),
            name: self.phpdoc_name(&import.imported_alias),
            from: self.resolve_phpdoc_class(&import.imported_from),
            r#as: import.imported_as.as_ref().map(|imported_as| self.phpdoc_name(&imported_as.local)),
        }
    }

    pub(crate) fn lower_method_annotation(
        &self,
        method: &'arena MethodTagValue<'arena>,
    ) -> MethodAnnotation<'arena, (), (), ()> {
        let type_parameters: &'arena [_] = match method.templates {
            Some(templates) => self.arena.alloc_slice_fill_iter(
                templates
                    .entries
                    .iter()
                    .map(|entry| self.lower_type_parameter_annotation(&entry.template, Variance::Invariant)),
            ),
            None => &[],
        };

        let parameters = self.arena.alloc_slice_fill_iter(
            method.parameters.entries.iter().map(|parameter| self.lower_parameter_annotation(parameter)),
        );

        MethodAnnotation {
            span: method.span(),
            r#static: method.r#static.is_some(),
            name: self.phpdoc_name(&method.name),
            type_parameters,
            parameters,
            return_type: method.return_type.map(|return_type| self.lower_type_annotation(return_type)),
        }
    }

    pub(crate) fn lower_property_annotation(
        &self,
        property: &'arena PropertyTagValue<'arena>,
        kind: PropertyAnnotationKind,
    ) -> PropertyAnnotation<'arena> {
        PropertyAnnotation {
            span: property.span(),
            kind,
            r#type: Some(self.lower_type_annotation(property.r#type)),
            variable: DirectVariable { span: property.variable.span, name: property.variable.value },
        }
    }
}
