<?php

declare(strict_types=1);

function accept_string(string $_): void
{
}

/**
 * @mago-expect analysis:possibly-null-operand
 * @mago-expect analysis:null-operand
 */
function test_loose_null_equality_empty_string(?string $s): string
{
    if ($s == null) {
        if ($s === null) {
            return "hello";
        }

        accept_string($s);
    } else {
        accept_string($s);
    }

    return $s;
}

/**
 * @mago-expect analysis:possibly-null-operand
 * @mago-expect analysis:null-operand
 */
function test_loose_null_inequality_retains_string(?string $s): string
{
    if ($s != null) {
        accept_string($s);
        return $s;
    }

    return "default";
}

/**
 * @mago-expect analysis:possibly-null-operand
 * @mago-expect analysis:null-operand
 */
function test_negative_invalid_argument(?string $s): void
{
    if ($s == null) {
        if ($s === null) {
            return;
        }

        // @mago-expect analysis:invalid-argument
        accept_string(123);
    }
}

/** @param 0 $_ */
function accept_zero(int $_): void {}

/** @param 0.0 $_ */
function accept_float_zero(float $_): void {}

/** @param '' $_ */
function accept_empty_string(string $_): void {}

/** @param non-empty-string $_ */
function accept_non_empty_string(string $_): void {}

/** @param null|false|0|0.0|''|array{} $_ */
function accept_null_equivalent(mixed $_): void {}

function test_nullable_integer(?int $value): void
{
    // @mago-expect analysis:possibly-null-operand,null-operand
    if ($value == null) {
        if ($value === null) {
            return;
        }

        accept_zero($value);
    }
}

function test_nullable_float(?float $value): void
{
    // @mago-expect analysis:possibly-null-operand,null-operand
    if (null == $value) {
        if ($value === null) {
            return;
        }

        accept_float_zero($value);
    }
}

function test_mixed(mixed $value): void
{
    // @mago-expect analysis:mixed-operand,null-operand
    if ($value == null) {
        accept_null_equivalent($value);

        if ($value === null) {
            return;
        }

        accept_null_equivalent($value);
    }
}

/** @param scalar $value */
function test_scalar(mixed $value): void
{
    // @mago-expect analysis:null-operand
    if ($value == null) {
        accept_null_equivalent($value);

        if (is_int($value)) {
            accept_zero($value);
        } else {
            accept_null_equivalent($value);
        }
    }
}

/** @param array-key $value */
function test_array_key(int|string $value): void
{
    // @mago-expect analysis:null-operand
    if ($value == null) {
        if (is_int($value)) {
            accept_zero($value);
        } else {
            accept_empty_string($value);
        }
    }
}

function test_non_empty_string(?string $value): void
{
    // @mago-expect analysis:possibly-null-operand,null-operand
    if ($value == null) {
        if ($value !== null) {
            accept_empty_string($value);
        }
    } else {
        accept_non_empty_string($value);
    }

    // @mago-expect analysis:possibly-null-operand,null-operand
    if ($value != null) {
        accept_non_empty_string($value);
    }

    // @mago-expect analysis:possibly-null-operand,null-operand
    if (null <> $value) {
        accept_non_empty_string($value);
    }
}

function test_coalesced_string(?string $value): void
{
    // @mago-expect analysis:possibly-null-operand,null-operand
    if (($value ?? null) == null) {
        if ($value === null) {
            return;
        }

        accept_empty_string($value);
    } else {
        accept_non_empty_string($value);
    }
}

/** @param null|'0'|'' $value */
function test_literal_zero_string(?string $value): void
{
    // @mago-expect analysis:possibly-null-operand,null-operand
    if ($value != null) {
        // @mago-expect analysis:invalid-argument
        accept_empty_string($value);
    }
}

/** @param false $_ */
function accept_false(bool $_): void {}

/** @param true $_ */
function accept_true(bool $_): void {}

/** @param array{} $_ */
function accept_empty_array(array $_): void {}

/** @param non-empty-array<int, string> $_ */
function accept_non_empty_array(array $_): void {}

function test_nullable_boolean(?bool $value): void
{
    // @mago-expect analysis:possibly-null-operand,null-operand
    if ($value == null) {
        if ($value === null) {
            return;
        }

        accept_false($value);
    } else {
        accept_true($value);
    }
}

/** @param array<int, string>|null $value */
function test_nullable_array(?array $value): void
{
    // @mago-expect analysis:possibly-null-operand,null-operand
    if ($value == null) {
        if ($value === null) {
            return;
        }

        accept_empty_array($value);
    } else {
        accept_non_empty_array($value);
    }
}

function test_negated_equality(?string $value): void
{
    // @mago-expect analysis:possibly-null-operand,null-operand
    if (!($value == null)) {
        accept_non_empty_string($value);
    }
}

/** @param array{value?: string|null} $array */
function test_optional_array_value(array $array): void
{
    // @mago-expect analysis:possibly-null-operand,null-operand
    if (($array['value'] ?? null) == null) {
        if (isset($array['value'])) {
            accept_empty_string($array['value']);
        }
    } else {
        accept_non_empty_string($array['value']);
    }
}

function test_strict_null_comparisons(?string $value): void
{
    if ($value === null) {
        // @mago-expect analysis:null-argument
        accept_string($value);
    } else {
        accept_string($value);
        // @mago-expect analysis:possibly-invalid-argument
        accept_non_empty_string($value);
    }
}

/** @param 0.0|0.00000000000000001|null $value */
function test_small_float(?float $value): void
{
    // @mago-expect analysis:possibly-null-operand,null-operand
    if ($value == null) {
        if ($value === null) {
            return;
        }

        accept_float_zero($value);
    } else {
        // @mago-expect analysis:invalid-argument
        accept_float_zero($value);
    }
}

function accept_null(null $_): void {}

/** @param 0.00000000000000001|null $value */
function test_nonzero_float(?float $value): void
{
    // @mago-expect analysis:possibly-null-operand,null-operand
    if ($value == null) {
        accept_null($value);
    }
}

/** @param non-empty-string|null $value */
function test_non_empty_string_or_null(?string $value): void
{
    // @mago-expect analysis:possibly-null-operand,null-operand
    if ($value == null) {
        accept_null($value);
    }
}

/** @param numeric-string|null $value */
function test_numeric_string_or_null(?string $value): void
{
    // @mago-expect analysis:possibly-null-operand,null-operand
    if ($value == null) {
        accept_null($value);
    }
}

/** @param literal-float|null $value */
function test_unknown_literal_float(?float $value): void
{
    // @mago-expect analysis:possibly-null-operand,null-operand
    if ($value == null) {
        if ($value === null) {
            return;
        }

        accept_float_zero($value);
    }
}

/** @param class-string|null $value */
function test_class_string_or_null(?string $value): void
{
    // @mago-expect analysis:possibly-null-operand,null-operand
    if ($value == null) {
        accept_null($value);
    }
}

/** @param non-empty-array<int, string>|null $value */
function test_non_empty_array_or_null(?array $value): void
{
    // @mago-expect analysis:possibly-null-operand,null-operand
    if ($value == null) {
        accept_null($value);
    }
}

/**
 * @template T of string|null
 * @param T $value
 */
function test_template_string(?string $value): void
{
    // @mago-expect analysis:possibly-null-operand,null-operand
    if ($value == null) {
        if ($value === null) {
            return;
        }

        accept_empty_string($value);
    } else {
        accept_non_empty_string($value);
    }
}
