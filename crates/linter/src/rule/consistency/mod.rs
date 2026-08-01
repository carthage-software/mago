pub mod ambiguous_constant_access;
pub mod ambiguous_function_call;
pub mod array_style;
pub mod assertion_style;
pub mod block_statement;
pub mod braced_string_interpolation;
pub mod class_name;
pub mod constant_name;
pub mod enum_name;
pub mod file_name;
pub mod function_name;
pub mod interface_name;
pub mod lowercase_keyword;
pub mod lowercase_type_hint;
pub mod method_name;
pub mod no_alias_function;
pub mod no_alternative_syntax;
pub mod no_fully_qualified_global_class_like;
pub mod no_fully_qualified_global_constant;
pub mod no_fully_qualified_global_function;
pub mod no_hash_comment;
pub mod no_php_tag_terminator;
pub mod no_trailing_space;
pub mod property_name;
pub mod string_style;
pub mod trait_name;
pub mod variable_name;

use mago_syntax_core::part_of_identifier;
use mago_syntax_core::start_of_identifier;

fn is_valid_identifier(identifier: &[u8]) -> bool {
    let Some((first, rest)) = identifier.split_first() else {
        return false;
    };

    matches!(*first, start_of_identifier!()) && rest.iter().all(|byte| matches!(*byte, part_of_identifier!()))
}

pub use ambiguous_constant_access::*;
pub use ambiguous_function_call::*;
pub use array_style::*;
pub use assertion_style::*;
pub use block_statement::*;
pub use braced_string_interpolation::*;
pub use class_name::*;
pub use constant_name::*;
pub use enum_name::*;
pub use file_name::*;
pub use function_name::*;
pub use interface_name::*;
pub use lowercase_keyword::*;
pub use lowercase_type_hint::*;
pub use method_name::*;
pub use no_alias_function::*;
pub use no_alternative_syntax::*;
pub use no_fully_qualified_global_class_like::*;
pub use no_fully_qualified_global_constant::*;
pub use no_fully_qualified_global_function::*;
pub use no_hash_comment::*;
pub use no_php_tag_terminator::*;
pub use no_trailing_space::*;
pub use property_name::*;
pub use string_style::*;
pub use trait_name::*;
pub use variable_name::*;

#[cfg(test)]
mod tests {
    use super::ClassNameRule;
    use super::EnumNameRule;
    use super::FunctionNameRule;
    use super::InterfaceNameRule;
    use super::MethodNameRule;
    use super::PropertyNameRule;
    use super::TraitNameRule;
    use super::VariableNameRule;
    use crate::test_lint_success;

    test_lint_success! {
        name = class_name_does_not_suggest_invalid_identifier,
        rule = ClassNameRule,
        code = "<?php class _360_class {}",
    }

    test_lint_success! {
        name = interface_name_does_not_suggest_invalid_identifier,
        rule = InterfaceNameRule,
        code = "<?php interface _360_interface {}",
    }

    test_lint_success! {
        name = trait_name_does_not_suggest_invalid_identifier,
        rule = TraitNameRule,
        code = "<?php trait _360_trait {}",
    }

    test_lint_success! {
        name = enum_name_does_not_suggest_invalid_identifier,
        rule = EnumNameRule,
        code = "<?php enum _360_enum {}",
    }

    test_lint_success! {
        name = function_name_does_not_suggest_invalid_identifier,
        rule = FunctionNameRule,
        code = "<?php function _360_FUNCTION(): void {}",
    }

    test_lint_success! {
        name = method_name_does_not_suggest_invalid_identifier,
        rule = MethodNameRule,
        code = "<?php class Example { public function _360_METHOD(): void {} }",
    }

    test_lint_success! {
        name = property_name_does_not_suggest_invalid_identifier,
        rule = PropertyNameRule,
        code = "<?php class Example { public string $_360_PROPERTY; }",
    }

    test_lint_success! {
        name = variable_name_does_not_suggest_invalid_identifier,
        rule = VariableNameRule,
        code = "<?php $___360 = 1;",
    }
}
