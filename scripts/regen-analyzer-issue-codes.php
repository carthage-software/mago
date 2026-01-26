#!/usr/bin/env php
<?php

declare(strict_types=1);

/**
 * A script to regenerate the Rust source code for the `IssueCode` enum and its implementations.
 */
namespace Mago\Scripts;

use function count;
use function ksort;
use function str_replace;
use function ucwords;

/**
 * Generates the Rust source code for the `IssueCode` enum and its implementations.
 *
 * This utility class is used internally to keep the list of issue codes
 * in sync without manual editing of Rust files.
 *
 * @mago-expect lint:kan-defect
 * @mago-expect lint:cyclomatic-complexity
 */
final class AnalyzerCodeModuleGenerator
{
    private const CODE_VALUES = [
        'invalid-assignment',
        'assignment-to-this',
        'assignment-to-constant',
        'abstract-instantiation',
        'clone-inside-loop',
        'duplicate-array-key',
        'falsable-return-statement',
        'impossible-array-access',
        'impossible-array-assignment',
        'mixed-assignment',
        'impossible-assignment',
        'missing-magic-method',
        'dynamic-static-method-call',
        'impossible-key-check',
        'impossible-nonnull-entry-check',
        'impossible-null-type-comparison',
        'impossible-condition',
        'impossible-type-comparison',
        'invalid-docblock',
        'invalid-argument',
        'invalid-array-element-key',
        'invalid-array-element',
        'mismatched-array-index',
        'invalid-array-index',
        'invalid-array-access',
        'invalid-method-access',
        'invalid-property-assignment-value',
        'invalid-continue',
        'invalid-break',
        'invalid-return-statement',
        'invalid-type-cast',
        'invalid-global',
        'invalid-throw',
        'invalid-unset',
        'invalid-callable',
        'invalid-named-argument',
        'less-specific-argument',
        'less-specific-nested-argument-type',
        'less-specific-nested-return-statement',
        'less-specific-return-statement',
        'method-access-on-null',
        'mixed-return-statement',
        'mixed-argument',
        'mixed-array-access',
        'mixed-array-assignment',
        'mixed-array-index',
        'mixed-method-access',
        'possibly-null-operand',
        'null-operand',
        'possibly-false-operand',
        'false-operand',
        'possibly-invalid-operand',
        'invalid-operand',
        'mixed-operand',
        'array-append-in-read-context',
        'missing-constructor',
        'missing-return-statement',
        'mixed-property-type-coercion',
        'no-value',
        'non-existent-class',
        'non-existent-class-like',
        'non-existent-constant',
        'non-existent-function',
        'non-existent-method',
        'non-existent-property',
        'possibly-non-existent-property',
        'possibly-non-existent-method',
        'never-return',
        'null-array-index',
        'nullable-return-statement',
        'paradoxical-condition',
        'possible-method-access-on-null',
        'possibly-invalid-argument',
        'null-array-access',
        'possibly-null-array-access',
        'possibly-null-array-index',
        'possibly-undefined-array-index',
        'possibly-undefined-int-array-index',
        'possibly-undefined-string-array-index',
        'property-type-coercion',
        'redundant-cast',
        'redundant-null-coalesce',
        'implicit-to-string-cast',
        'redundant-isset-check',
        'redundant-key-check',
        'redundant-nonnull-entry-check',
        'redundant-nonnull-type-comparison',
        'redundant-condition',
        'redundant-type-comparison',
        'redundant-comparison',
        'redundant-logical-operation',
        'redundant-docblock-type',
        'too-few-arguments',
        'too-many-arguments',
        'undefined-int-array-index',
        'undefined-string-array-index',
        'undefined-variable',
        'possibly-undefined-variable',
        'reference-to-undefined-variable',
        'unevaluated-code',
        'uninitialized-property',
        'unused-function-call',
        'unused-method-call',
        'unused-statement',
        'useless-control-flow',
        'impure-static-variable',
        'conflicting-template-equality-bounds',
        'incompatible-template-lower-bound',
        'deprecated-function',
        'deprecated-method',
        'deprecated-closure',
        'named-argument-not-allowed',
        'duplicate-named-argument',
        'named-argument-overrides-positional',
        'named-argument-after-positional',
        'template-constraint-violation',
        'array-to-string-conversion',
        'implicit-resource-to-string-cast',
        'match-expression-only-default-arm',
        'empty-match-expression',
        'unknown-match-subject-type',
        'unreachable-match-arm',
        'unreachable-match-default-arm',
        'match-arm-always-true',
        'match-default-arm-always-executed',
        'match-subject-type-is-never',
        'match-not-exhaustive',
        'non-existent-attribute-class',
        'non-class-used-as-attribute',
        'class-not-marked-as-attribute',
        'attribute-not-repeatable',
        'abstract-class-used-as-attribute',
        'invalid-attribute-target',
        'invalid-catch-type',
        'duplicate-caught-type',
        'no-valid-catch-type-found',
        'catch-type-not-throwable',
        'invalid-catch-type-not-class-or-interface',
        'non-existent-catch-type',
        'unknown-iterator-type',
        'null-iterator',
        'possibly-null-iterator',
        'false-iterator',
        'possibly-false-iterator',
        'generic-object-iteration',
        'non-iterable-object-iteration',
        'enum-iteration',
        'invalid-iterator',
        'possibly-invalid-iterator',
        'invalid-foreach-key',
        'invalid-foreach-value',
        'undefined-variable-in-closure-use',
        'duplicate-closure-use-variable',
        'duplicate-definition',
        'invalid-yield-value-type',
        'invalid-yield-key-type',
        'unknown-yield-from-iterator-type',
        'yield-from-invalid-send-type',
        'yield-from-non-iterable',
        'yield-from-invalid-value-type',
        'yield-from-invalid-key-type',
        'yield-outside-function',
        'invalid-generator-return-type',
        'hidden-generator-return',
        'name-already-in-use',
        'self-outside-class-scope',
        'static-outside-class-scope',
        'parent-outside-class-scope',
        'invalid-parent-type',
        'invalid-class-string-expression',
        'unknown-class-instantiation',
        'interface-instantiation',
        'trait-instantiation',
        'enum-instantiation',
        'deprecated-class',
        'deprecated-trait',
        'unsafe-instantiation',
        'ambiguous-instantiation-target',
        'unknown-constant-selector-type',
        'string-constant-selector',
        'invalid-constant-selector',
        'ambiguous-class-like-constant-access',
        'invalid-class-constant-on-string',
        'unknown-member-selector-type',
        'string-member-selector',
        'invalid-member-selector',
        'invalid-static-method-call',
        'deprecated-feature',
        'type-inspection',
        'type-confirmation',
        'null-property-access',
        'possibly-null-property-access',
        'mixed-property-access',
        'invalid-property-access',
        'ambiguous-object-property-access',
        'invalid-static-property-access',
        'redundant-nullsafe-operator',
        'ambiguous-object-method-access',
        'impure-construct',
        'docblock-type-mismatch',
        'invalid-destructuring-source',
        'mixed-destructuring-shape',
        'skip-in-keyed-destructuring',
        'spread-in-destructuring',
        'invalid-static-method-access',
        'possibly-static-access-on-interface',
        'static-access-on-interface',
        'list-used-in-read-context',
        'invalid-scope-keyword-context',
        'mixed-clone',
        'possibly-invalid-clone',
        'invalid-clone',
        'invalid-extend',
        'invalid-implement',
        'missing-required-interface',
        'missing-required-parent',
        'missing-template-parameter',
        'excess-template-parameter',
        'inconsistent-template',
        'invalid-template-parameter',
        'unimplemented-abstract-method',
        'unimplemented-abstract-property-hook',
        'incompatible-static-modifier',
        'incompatible-readonly-modifier',
        'incompatible-visibility',
        'incompatible-parameter-count',
        'incompatible-parameter-type',
        'incompatible-return-type',
        'incompatible-parameter-name',
        'non-static-abstract-implementation',
        'incompatible-property-access',
        'incompatible-property-type',
        'invalid-property-write',
        'invalid-property-read',
        'deprecated-constant',
        'invalid-enum-case-value',
        'condition-is-too-complex',
        'expression-is-too-complex',
        'where-constraint-violation',
        'write-only-property',
        'extend-final-class',
        'non-existent-class-constant',
        'possibly-false-argument',
        'false-argument',
        'possibly-null-argument',
        'null-argument',
        'never-matching-switch-case',
        'always-matching-switch-case',
        'unreachable-switch-case',
        'unreachable-switch-default',
        'invalid-isset-expression',
        'invalid-trait-use',
        'trait-constant-override',
        'incompatible-constant-override',
        'direct-trait-constant-access',
        'override-final-constant',
        'incompatible-constant-access',
        'incompatible-constant-type',
        'incompatible-constant-visibility',
        'incompatible-property-override',
        'incompatible-property-visibility',
        'incompatible-property-static',
        'incompatible-property-readonly',
        'incompatible-property-default',
        'incompatible-property-hook-parameter-type',
        'incompatible-property-hook-signature',
        'backed-property-reference-hook',
        'override-final-method',
        'override-final-property',
        'override-final-property-hook',
        'invalid-trait-alias-modifier',
        'psalm-trace',
        'reference-constraint-violation',
        'invalid-pass-by-reference',
        'conflicting-reference-constraint',
        'unhandled-thrown-type',
        'avoid-catching-error',
        'missing-override-attribute',
        'invalid-override-attribute',
        'unused-parameter',
        'unused-template-parameter',
        'unused-method',
        'unused-property',
        'reference-reused-from-confusing-scope',
        'non-documented-method',
        'non-documented-property',
        'non-documented-constant',
        'missing-parameter-type',
        'missing-property-type',
        'missing-constant-type',
        'missing-return-type',
        'invalid-array-access-assignment-value',
        'non-existent-use-import',
    ];

    /**
     * @param array<string, string> $allCodes
     */
    private function __construct(
        private readonly array $allCodes,
    ) {}

    public static function generate(): string
    {
        $instance = self::fromRawData();

        $enum = $instance->generateEnum();
        $impl = $instance->generateImpl();
        $traits = $instance->generateTraits();

        return $enum . $impl . $traits;
    }

    private static function fromRawData(): self
    {
        $allCodes = [];
        foreach (self::CODE_VALUES as $value) {
            $pascalCase = str_replace('-', '', ucwords($value, '-'));
            $allCodes[$pascalCase] = $value;
        }

        ksort($allCodes);
        return new self($allCodes);
    }

    private function generateEnum(): string
    {
        if (count($this->allCodes) > 65535) {
            throw new \RuntimeException('Too many issue codes; cannot be represented by a u16.');
        }

        $enum = '//! This file is auto-generated. Do not edit manually.' . "\n\n";
        $enum .= "/// An issue code representing a specific type of analysis issue.\n";
        $enum .= "///\n";
        $enum .= "/// Each issue code corresponds to a unique identifier for a particular\n";
        $enum .= "/// kind of issue that the analyzer can detect during code analysis.\n";
        $enum .= "///\n";
        $enum .= "/// This enum is non-exhaustive; new issue codes may be added in future versions.\n";
        $enum .= "#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n";
        $enum .= "#[repr(u16)]\n";
        $enum .= "#[non_exhaustive]\n";
        $enum .= "pub enum IssueCode {\n";
        foreach ($this->allCodes as $code => $_) {
            $enum .= "    {$code},\n";
        }
        $enum .= "}\n\n";
        return $enum;
    }

    private function generateImpl(): string
    {
        $impl = "impl IssueCode {\n";

        // as_str() method
        $impl .= "    pub fn as_str(&self) -> &'static str {\n";
        $impl .= "        match self {\n";
        foreach ($this->allCodes as $code => $value) {
            $impl .= "            Self::{$code} => \"{$value}\",\n";
        }
        $impl .= "        }\n    }\n\n";

        // as_u16() method
        $impl .= "    pub fn as_u16(&self) -> u16 {\n";
        $impl .= "        *self as u16\n";
        $impl .= "    }\n\n";

        return $impl . "}\n\n";
    }

    private function generateTraits(): string
    {
        // FromStr trait
        $fromStr = "impl std::str::FromStr for IssueCode {\n";
        $fromStr .= "    type Err = &'static str;\n\n";
        $fromStr .= "    fn from_str(s: &str) -> Result<Self, Self::Err> {\n";
        $fromStr .= "        match s {\n";
        foreach ($this->allCodes as $code => $value) {
            $fromStr .= "            \"{$value}\" => Ok(Self::{$code}),\n";
        }
        $fromStr .= "            _ => Err(\"unknown issue code\"),\n";
        $fromStr .= "        }\n    }\n}\n\n";

        // Boilerplate trait impls
        $boilerplate = <<<RUST
            impl std::fmt::Display for IssueCode {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "{}", self.as_str())
                }
            }

            impl std::convert::TryFrom<&str> for IssueCode {
                type Error = &'static str;

                fn try_from(value: &str) -> Result<Self, Self::Error> {
                    <Self as std::str::FromStr>::from_str(value)
                }
            }

            impl std::convert::From<IssueCode> for &'static str {
                fn from(val: IssueCode) -> Self {
                    val.as_str()
                }
            }

            impl std::convert::From<IssueCode> for String {
                fn from(val: IssueCode) -> Self {
                    val.as_str().to_string()
                }
            }

            impl std::borrow::Borrow<str> for IssueCode {
                fn borrow(&self) -> &'static str {
                    self.as_str()
                }
            }

            impl<'a> std::borrow::Borrow<str> for &'a IssueCode {
                fn borrow(&self) -> &'a str {
                    self.as_str()
                }
            }

            RUST;

        return $fromStr . $boilerplate;
    }
}

echo AnalyzerCodeModuleGenerator::generate();
