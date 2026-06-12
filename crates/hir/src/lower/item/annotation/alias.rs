use mago_allocator::Arena;
use mago_phpdoc_syntax::cst::tag::TypeAliasImportTagValue;
use mago_phpdoc_syntax::cst::tag::TypeAliasTagValue;
use mago_span::HasSpan;

use crate::ir::item::annotation::alias::ImportedTypeAliasAnnotation;
use crate::ir::item::annotation::alias::TypeAliasAnnotation;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_type_alias_annotation(
        &mut self,
        alias: &'scratch TypeAliasTagValue<'scratch>,
    ) -> TypeAliasAnnotation<'arena> {
        TypeAliasAnnotation {
            span: alias.span(),
            name: self.phpdoc_name(&alias.alias),
            r#type: self.lower_type_annotation(alias.r#type),
        }
    }

    pub(crate) fn lower_imported_type_alias_annotation(
        &mut self,
        import: &'scratch TypeAliasImportTagValue<'scratch>,
    ) -> ImportedTypeAliasAnnotation<'arena> {
        ImportedTypeAliasAnnotation {
            span: import.span(),
            name: self.phpdoc_name(&import.imported_alias),
            from: self.resolve_phpdoc_class(&import.imported_from),
            r#as: import.imported_as.as_ref().map(|imported_as| self.phpdoc_name(&imported_as.local)),
        }
    }
}
