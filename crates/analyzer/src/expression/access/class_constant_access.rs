use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_never;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::resolver::class_constant::resolve_class_constants;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for ClassConstantAccess<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let resolution = resolve_class_constants(context, block_context, artifacts, self.class, &self.constant, false)?;

        let mut resulting_type = if resolution.has_ambiguous_path { Some(get_mixed()) } else { None };
        for resolved_constant in resolution.constants {
            resulting_type =
                Some(add_optional_union_type(resolved_constant.const_type, resulting_type.as_ref(), context.codebase));
        }

        artifacts.set_expression_type(
            self,
            if resolution.has_invalid_path { get_never() } else { resulting_type.unwrap_or_else(get_mixed) },
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = const_access_on_undefined_class,
        code = indoc! {r#"
            <?php

            $_ = NonExistentClass::SOME_CONST;
        "#},
        issues = [
            IssueCode::NonExistentClassLike,
            IssueCode::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = const_access_self_outside_class,
        code = indoc! {r#"
            <?php

            $_ = self::SOME_CONST;
        "#},
        issues = [
            IssueCode::SelfOutsideClassScope,
            IssueCode::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = const_access_static_outside_class,
        code = indoc! {r#"
            <?php

            $_ = static::SOME_CONST;
        "#},
        issues = [
            IssueCode::StaticOutsideClassScope,
            IssueCode::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = const_access_parent_outside_class,
        code = indoc! {r#"
            <?php

            $_ = parent::SOME_CONST;
        "#},
        issues = [
            IssueCode::ParentOutsideClassScope,
            IssueCode::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = const_access_dynamic_class_unknown_type,
        code = indoc! {r#"
            <?php

            $const = $unknownVar::{KNOWN_CONST};
        "#},
        issues = [
            IssueCode::UndefinedVariable,
            IssueCode::NonExistentConstant,
            IssueCode::UnknownConstantSelectorType,
            IssueCode::MixedAssignment,
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
            IssueCode::UndefinedVariable,
            IssueCode::InvalidConstantSelector,
            IssueCode::MixedAssignment,
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
            IssueCode::StringConstantSelector,
            IssueCode::MixedAssignment,
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
            IssueCode::InvalidConstantSelector,
            IssueCode::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = const_access_class_on_string_variable,
        code = indoc! {r#"
            <?php
            $className = "MyClass";
            $_ = $className::class;
        "#},
        issues = [
            IssueCode::InvalidClassConstantOnString,
            IssueCode::ImpossibleAssignment,
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
            IssueCode::AmbiguousClassLikeConstantAccess,
            IssueCode::MixedAssignment,
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
        issues = [
            IssueCode::AmbiguousClassLikeConstantAccess,
        ]
    }

    test_analysis! {
        name = const_access_undefined_on_enum,
        code = indoc! {r#"
            <?php
            enum Suit { case Hearts; }
            $_ = Suit::Diamonds; // Accessing 'Diamonds' like a const/case
        "#},
        issues = [
            IssueCode::NonExistentClassConstant,
            IssueCode::ImpossibleAssignment,
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

    test_analysis! {
        name = const_type_inference,
        code = indoc! {r#"
            <?php

            class A
            {
                public const FOO = <<<'XML'
                    <?xml version="1.0" encoding="UTF-8"?>
                    <foo>
                        <bar>baz</bar>
                    </foo>
                XML;

                /**
                 * @return non-empty-string
                 */
                public function getFoo(): string
                {
                    return self::FOO;
                }
            }
        "#},
    }

    test_analysis! {
        name = const_var_type,
        code = indoc! {r#"
            <?php

            class NumberFormatter
            {
                public const int CURRENCY = 1;
                public const int DECIMAL = 2;

                private string $locale;
                private int $style;

                public function __construct(string $locale, int $style)
                {
                    $this->locale = $locale;
                    $this->style = $style;
                }

                public function formatCurrency(float $_value, string $_currency): string|false
                {
                    return false;
                }
            }

            enum PremiumBasisCode: string
            {
                case ADMISSIONS = 'admissions';
                case AREA = 'area';
                case COST = 'cost';
                case EACH = 'each';
                case FLAT = 'flat';
                case OTHER = 'other';
                case PAYROLL = 'payroll';
                case RECEIPTS = 'receipts';
                case SALES = 'sales';
                case UNITS = 'units';
            }

            class ExposureFormatter
            {
                /**
                 * @var array<string, int>
                 */
                private const array PREFIXES = [
                    PremiumBasisCode::ADMISSIONS->value => NumberFormatter::CURRENCY,
                    PremiumBasisCode::AREA->value => NumberFormatter::DECIMAL,
                    PremiumBasisCode::COST->value => NumberFormatter::CURRENCY,
                    PremiumBasisCode::EACH->value => NumberFormatter::DECIMAL,
                    PremiumBasisCode::FLAT->value => NumberFormatter::CURRENCY,
                    PremiumBasisCode::OTHER->value => NumberFormatter::DECIMAL,
                    PremiumBasisCode::PAYROLL->value => NumberFormatter::CURRENCY,
                    PremiumBasisCode::RECEIPTS->value => NumberFormatter::CURRENCY,
                    PremiumBasisCode::SALES->value => NumberFormatter::CURRENCY,
                    PremiumBasisCode::UNITS->value => NumberFormatter::DECIMAL,
                ];

                private string $locale;

                public function __construct(string $locale = 'en_US')
                {
                    $this->locale = $locale;
                }

                public function format(PremiumBasisCode $basisCode, float $value): null|string
                {
                    $formatter = new NumberFormatter($this->locale, self::PREFIXES[$basisCode->value]);
                    $result = $formatter->formatCurrency($value, 'USD');

                    if (false === $result) {
                        return null;
                    } else {
                        return $result;
                    }
                }
            }
        "#},
    }
}
