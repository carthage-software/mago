use mago_atom::Atom;

use crate::ttype::atomic::TAtomic;
use crate::ttype::template::TemplateBound;
use crate::ttype::union::TUnion;

mod callable_comparator;
mod class_string_comparator;
mod derived_comparator;
mod generic_comparator;
mod integer_comparator;
mod iterable_comparator;
mod resource_comparator;
mod scalar_comparator;

pub(super) mod array_comparator;
pub(super) mod object_comparator;

pub mod atomic_comparator;
pub mod union_comparator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonResult {
    pub type_coerced: Option<bool>,
    pub type_coerced_from_nested_mixed: Option<bool>,
    pub type_coerced_from_as_mixed: Option<bool>,
    pub type_coerced_to_literal: Option<bool>,
    pub replacement_union_type: Option<TUnion>,
    pub replacement_atomic_type: Option<TAtomic>,
    pub type_variable_lower_bounds: Vec<(Atom, TemplateBound)>,
    pub type_variable_upper_bounds: Vec<(Atom, TemplateBound)>,
}

impl Default for ComparisonResult {
    fn default() -> Self {
        Self::new()
    }
}

impl ComparisonResult {
    pub fn new() -> Self {
        Self {
            type_coerced: None,
            type_coerced_from_nested_mixed: None,
            type_coerced_from_as_mixed: None,
            type_coerced_to_literal: None,
            replacement_union_type: None,
            replacement_atomic_type: None,
            type_variable_lower_bounds: vec![],
            type_variable_upper_bounds: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use bumpalo::Bump;
    use mago_atom::atom;
    use mago_database::Database;
    use mago_database::DatabaseReader;
    use mago_database::file::File;
    use mago_names::resolver::NameResolver;
    use mago_syntax::parser::parse_file;

    use crate::metadata::CodebaseMetadata;
    use crate::populator::populate_codebase;
    use crate::reference::SymbolReferences;
    use crate::scanner::scan_program;
    use crate::ttype::atomic::TAtomic;
    use crate::ttype::atomic::object::TObject;
    use crate::ttype::comparator::ComparisonResult;
    use crate::ttype::comparator::union_comparator::is_contained_by;
    use crate::ttype::union::TUnion;

    pub(crate) fn create_test_codebase(code: &'static str) -> CodebaseMetadata {
        let file = File::ephemeral(Cow::Borrowed("code.php"), Cow::Borrowed(code));
        let database = Database::single(file);

        let mut codebase = CodebaseMetadata::new();
        let arena = Bump::new();
        for file in database.files() {
            let program = parse_file(&arena, &file).0;
            let resolved_names = NameResolver::new(&arena).resolve(program);
            let program_codebase = scan_program(&arena, &file, program, &resolved_names);

            codebase.extend(program_codebase);
        }

        populate_codebase(&mut codebase, &mut SymbolReferences::new(), Default::default(), Default::default());

        codebase
    }

    pub(crate) fn assert_is_contained_by(
        codebase: &CodebaseMetadata,
        input: &TUnion,
        container: &TUnion,
        expected: bool,
        comparison_result: &mut ComparisonResult,
    ) {
        let is_contained_by = is_contained_by(codebase, input, container, false, false, false, comparison_result);

        assert_eq!(is_contained_by, expected);
    }

    #[test]
    fn test_order_is_not_important() {
        let code = r#"
            <?php

            interface DateTimeInterface {}

            class DateTime implements DateTimeInterface {}
        "#;

        let codebase = create_test_codebase(code);

        let datetime_interface_type = TUnion::from_vec(vec![TAtomic::Object(TObject::new_named(atom("DateTime")))]);
        let datetime_interface_null_type =
            TUnion::from_vec(vec![TAtomic::Object(TObject::new_named(atom("DateTimeInterface"))), TAtomic::Null]);
        let null_datetime_interface_type =
            TUnion::from_vec(vec![TAtomic::Null, TAtomic::Object(TObject::new_named(atom("DateTimeInterface")))]);

        let mut first_comparison_result = ComparisonResult::new();
        let mut second_comparison_result = ComparisonResult::new();

        let first_is_contained_by = is_contained_by(
            &codebase,
            &datetime_interface_null_type,
            &datetime_interface_type,
            false,
            false,
            false,
            &mut first_comparison_result,
        );

        let second_is_contained_by = is_contained_by(
            &codebase,
            &null_datetime_interface_type,
            &datetime_interface_type,
            false,
            false,
            false,
            &mut second_comparison_result,
        );

        assert!(!first_is_contained_by);
        assert!(!second_is_contained_by);

        assert_eq!(first_comparison_result.type_coerced, Some(true));
        assert_eq!(second_comparison_result.type_coerced, Some(true));

        assert_eq!(first_comparison_result, second_comparison_result);
    }

    #[test]
    fn test_union_order_with_multiple_coercible_types() {
        let code = r#"
            <?php

            interface A {}
            interface B {}

            class C implements A, B {}
        "#;

        let codebase = create_test_codebase(code);

        let c_type = TUnion::from_vec(vec![TAtomic::Object(TObject::new_named(atom("C")))]);
        let a_b_type = TUnion::from_vec(vec![
            TAtomic::Object(TObject::new_named(atom("A"))),
            TAtomic::Object(TObject::new_named(atom("B"))),
        ]);
        let b_a_type = TUnion::from_vec(vec![
            TAtomic::Object(TObject::new_named(atom("B"))),
            TAtomic::Object(TObject::new_named(atom("A"))),
        ]);

        let mut first_comparison_result = ComparisonResult::new();
        let mut second_comparison_result = ComparisonResult::new();

        let first_is_contained_by =
            is_contained_by(&codebase, &a_b_type, &c_type, false, false, false, &mut first_comparison_result);
        let second_is_contained_by =
            is_contained_by(&codebase, &b_a_type, &c_type, false, false, false, &mut second_comparison_result);

        assert!(!first_is_contained_by);
        assert!(!second_is_contained_by);
        assert_eq!(first_comparison_result.type_coerced, Some(true));
        assert_eq!(second_comparison_result.type_coerced, Some(true));
        assert_eq!(first_comparison_result, second_comparison_result);
    }

    #[test]
    fn test_union_order_with_non_coercible_types() {
        let code = r#"
            <?php

            class Foo {}
            class Bar {}
        "#;

        let codebase = create_test_codebase(code);

        let foo_type = TUnion::from_vec(vec![TAtomic::Object(TObject::new_named(atom("Foo")))]);
        let bar_null_type = TUnion::from_vec(vec![TAtomic::Object(TObject::new_named(atom("Bar"))), TAtomic::Null]);
        let null_bar_type = TUnion::from_vec(vec![TAtomic::Null, TAtomic::Object(TObject::new_named(atom("Bar")))]);

        let mut first_comparison_result = ComparisonResult::new();
        let mut second_comparison_result = ComparisonResult::new();

        let first_is_contained_by =
            is_contained_by(&codebase, &bar_null_type, &foo_type, false, false, false, &mut first_comparison_result);
        let second_is_contained_by =
            is_contained_by(&codebase, &null_bar_type, &foo_type, false, false, false, &mut second_comparison_result);

        assert!(!first_is_contained_by);
        assert!(!second_is_contained_by);
        assert_eq!(first_comparison_result.type_coerced, None);
        assert_eq!(second_comparison_result.type_coerced, None);
        assert_eq!(first_comparison_result, second_comparison_result);
    }

    #[test]
    fn test_union_order_with_mixed_coercion() {
        let code = r#"
            <?php

            interface ParentInterface {}
            class Child implements ParentInterface {}
            class Unrelated {}
        "#;

        let codebase = create_test_codebase(code);

        let child_type = TUnion::from_vec(vec![TAtomic::Object(TObject::new_named(atom("Child")))]);
        let parent_unrelated_type = TUnion::from_vec(vec![
            TAtomic::Object(TObject::new_named(atom("ParentInterface"))),
            TAtomic::Object(TObject::new_named(atom("Unrelated"))),
        ]);
        let unrelated_parent_type = TUnion::from_vec(vec![
            TAtomic::Object(TObject::new_named(atom("Unrelated"))),
            TAtomic::Object(TObject::new_named(atom("ParentInterface"))),
        ]);

        let mut first_comparison_result = ComparisonResult::new();
        let mut second_comparison_result = ComparisonResult::new();

        let first_is_contained_by = is_contained_by(
            &codebase,
            &parent_unrelated_type,
            &child_type,
            false,
            false,
            false,
            &mut first_comparison_result,
        );
        let second_is_contained_by = is_contained_by(
            &codebase,
            &unrelated_parent_type,
            &child_type,
            false,
            false,
            false,
            &mut second_comparison_result,
        );

        assert!(!first_is_contained_by);
        assert!(!second_is_contained_by);
        assert_eq!(first_comparison_result.type_coerced, Some(true));
        assert_eq!(second_comparison_result.type_coerced, Some(true));
        assert_eq!(first_comparison_result, second_comparison_result);
    }
}
