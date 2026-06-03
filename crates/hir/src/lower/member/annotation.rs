use bumpalo::collections::Vec;
use mago_phpdoc_syntax::cst::tag::MethodTagValue;
use mago_phpdoc_syntax::cst::tag::PropertyTagValue;
use mago_phpdoc_syntax::cst::tag::TypeAliasImportTagValue;
use mago_phpdoc_syntax::cst::tag::TypeAliasTagValue;
use mago_span::HasSpan;

use crate::ir::generics::TypeParameterDefiningEntity;
use crate::ir::generics::Variance;
use crate::ir::identifier::Identifier;
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
        &mut self,
        method: &'arena MethodTagValue<'arena>,
        owner: Identifier<'arena>,
    ) -> MethodAnnotation<'arena, (), (), ()> {
        let name = self.phpdoc_name(&method.name);
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::Method(owner, name));

        let mut type_parameters = Vec::new_in(self.arena);
        if let Some(templates) = method.templates {
            for entry in templates.entries.iter() {
                let annotation = self.lower_type_parameter_annotation(&entry.template, Variance::Invariant);
                self.type_resolution.add_template(entry.template.name.value, annotation.bound);
                type_parameters.push(annotation);
            }
        }

        let parameters = self.arena.alloc_slice_fill_iter(
            method.parameters.entries.iter().map(|parameter| self.lower_parameter_annotation(parameter)),
        );
        let return_type = method.return_type.map(|return_type| self.lower_type_annotation(return_type));

        self.type_resolution.leave_scope();

        MethodAnnotation {
            span: method.span(),
            r#static: method.r#static.is_some(),
            name,
            type_parameters: type_parameters.into_bump_slice(),
            parameters,
            return_type,
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
