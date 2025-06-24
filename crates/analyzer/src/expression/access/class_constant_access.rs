use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::get_never;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::resolver::class_constant::resolve_class_constants;

impl Analyzable for ClassConstantAccess {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let resolution =
            resolve_class_constants(context, block_context, artifacts, &self.class, &self.constant, false)?;

        let mut resulting_type = if resolution.has_ambiguous_path { Some(get_mixed()) } else { None };
        for resolved_constant in resolution.constants {
            resulting_type = Some(add_optional_union_type(
                resolved_constant.const_type,
                resulting_type.as_ref(),
                context.codebase,
                context.interner,
            ));
        }

        let resulting_type = if resolution.has_invalid_path {
            get_never()
        } else {
            match resulting_type {
                Some(t) => t,
                None => get_mixed_any(),
            }
        };

        artifacts.set_expression_type(self, resulting_type);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::issue::TypingIssueKind;
    use crate::test_analysis;

    test_analysis! {
        name = class_like_constant_access,
        code = indoc! {r#"
            <?php

            class A
            {
                public const int Foo = 1;
                public const string Bar = 'bar';
            }

            enum B
            {
                /**
                 * @var list<int>
                 */
                public const array Foo = [1, 2, 3];

                case Bar;
            }

            /**
             * @param A|B|class-string<A>|enum-string<B> $c
             * @param 'Foo'|'Bar' $const
             *
             * @return int|string|list<int>|B
             */
            function get_constant(string|object $c, string $const): int|string|array|B
            {
                $value = $c::{$const};

                return $value;
            }

            $_int = get_constant(A::class, 'Foo'); // int(1)
            $_string = get_constant(A::class, 'Bar'); // string(3) "bar"
            $_array = get_constant(B::class, 'Foo'); // array(3) { [0]=> int(1) [1]=> int(2) [2]=> int(3) }
            $_enum = get_constant(B::class, 'Bar'); // enum(B::Bar)
        "#},
    }

    test_analysis! {
        name = accessing_undefined_class_constant,
        code = indoc! {r#"
            <?php

            class A
            {
            }

            $_ = A::Foo;
        "#},
        issues = [
            TypingIssueKind::UndefinedClassLikeConstant,
            TypingIssueKind::ImpossibleAssignment,
        ],
    }

    test_analysis! {
        name = const_access_on_undefined_class,
        code = indoc! {r#"
            <?php

            $_ = NonExistentClass::SOME_CONST;
        "#},
        issues = [
            TypingIssueKind::NonExistentClassLike,
            TypingIssueKind::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = const_access_self_outside_class,
        code = indoc! {r#"
            <?php

            $_ = self::SOME_CONST;
        "#},
        issues = [
            TypingIssueKind::SelfOutsideClassScope,
            TypingIssueKind::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = const_access_static_outside_class,
        code = indoc! {r#"
            <?php

            $_ = static::SOME_CONST;
        "#},
        issues = [
            TypingIssueKind::StaticOutsideClassScope,
            TypingIssueKind::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = const_access_parent_outside_class,
        code = indoc! {r#"
            <?php

            $_ = parent::SOME_CONST;
        "#},
        issues = [
            TypingIssueKind::ParentOutsideClassScope,
            TypingIssueKind::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = const_access_dynamic_class_unknown_type,
        code = indoc! {r#"
            <?php

            $const = $unknownVar::{KNOWN_CONST};
        "#},
        issues = [
            TypingIssueKind::UndefinedVariable, // `$unknownVar` is not defined
            TypingIssueKind::NonExistentConstant, // `KNOWN_CONST` does not exist
            TypingIssueKind::UnknownConstantSelectorType, // `{KNOWN_CONST}` is not a valid class constant name
            TypingIssueKind::MixedAssignment, // Overall assignment is mixed
        ]
    }

    test_analysis! {
        name = const_access_dynamic_const_name_unknown_type,
        code = indoc! {r#"
            <?php

            class MyClass { const C = 1; }

            $const = MyClass::{$unknownConstName};
        "#},
        issues = [
            TypingIssueKind::UndefinedVariable,
            TypingIssueKind::InvalidConstantSelector,
            TypingIssueKind::MixedAssignment,
        ]
    }

    test_analysis! {
        name = const_access_dynamic_const_name_non_literal_string,
        code = indoc! {r#"
            <?php

            class MyClass { const GREETING = "hello"; }

            function getName(): string { return "GREETING"; }
            $constName = getName();

            $const = MyClass::{$constName};
        "#},
        issues = [
            TypingIssueKind::StringConstantSelector,
            TypingIssueKind::MixedAssignment,
        ]
    }

    test_analysis! {
        name = const_access_dynamic_const_name_invalid_type,
        code = indoc! {r#"
            <?php

            class MyClass { const C = 1; }
            $constName = 123;
            $_ = MyClass::{$constName};
        "#},
        issues = [
            TypingIssueKind::InvalidConstantSelector,
            TypingIssueKind::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = const_access_class_on_string_variable,
        code = indoc! {r#"
            <?php
            $className = "MyClass"; /
            $_ = $className::class;
        "#},
        issues = [
            TypingIssueKind::InvalidClassConstantOnString,
            TypingIssueKind::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = const_access_on_generic_object_type,
        code = indoc! {r#"
            <?php
            class stdClass {} // stub

            function get_some_object(): object { return new stdClass(); }

            $obj = get_some_object();
            $const = $obj::SOME_CONST;
        "#},
        issues = [
            TypingIssueKind::AmbiguousClassLikeConstantAccess,
            TypingIssueKind::MixedAssignment,
        ]
    }

    test_analysis! {
        name = const_access_on_generic_class_string,
        code = indoc! {r#"
            <?php
            /** @param class-string $cs */
            function process_class_string(string $cs) {
                return $cs::SOME_CONST;
            }
        "#},
        issues = [TypingIssueKind::AmbiguousClassLikeConstantAccess]
    }

    test_analysis! {
        name = const_access_undefined_on_enum,
        code = indoc! {r#"
            <?php
            enum Suit { case Hearts; }
            $_ = Suit::Diamonds; // Accessing 'Diamonds' like a const/case
        "#},
        issues = [
            TypingIssueKind::UndefinedClassLikeConstant,
            TypingIssueKind::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = const_access_interface_const_via_class,
        code = indoc! {r#"
            <?php

            interface MyInterface { const IFACE_CONST = "hello"; }
            class ImplementingClass implements MyInterface {}

            $_ = ImplementingClass::IFACE_CONST;
        "#},
    }

    test_analysis! {
        name = const_access_on_interface_directly,
        code = indoc! {r#"
            <?php
            interface ConstantsInterface { const MY_CONST = 42; }
            $_ = ConstantsInterface::MY_CONST;
        "#},
    }
}
