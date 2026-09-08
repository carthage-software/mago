<?php

declare(strict_types=1);

/**
 * @template T
 * @param T $expected
 * @psalm-assert-if-true =T $value
 */
function equals_when_enabled(mixed $value, mixed $expected, bool $enabled): bool
{
    return $value === $expected && $enabled;
}

/** @psalm-assert-if-false =null $value */
function is_non_null_or_disabled(?object $value, bool $enabled): bool
{
    return $value !== null || !$enabled;
}

/** @param null $_value */
function takes_null(mixed $_value): void {}

function true_assertions_narrow_only_the_true_branch(?object $value, bool $first, bool $second): object
{
    if (equals_when_enabled($value, null, $first) || equals_when_enabled($value, null, $second)) {
        takes_null($value);

        return new stdClass();
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $value;
}

function false_assertions_narrow_only_the_false_branch(?object $value, bool $first, bool $second): object
{
    if (is_non_null_or_disabled($value, $first) && is_non_null_or_disabled($value, $second)) {
        // @mago-expect analysis:nullable-return-statement,invalid-return-statement
        return $value;
    }

    takes_null($value);

    return new stdClass();
}

function combines_nested_conjunctions(
    mixed $value,
    int|object $int_or_object,
    int|bool $int_or_bool,
    float|object $float_or_object,
    float|bool $float_or_bool,
    bool $enabled,
): int|float {
    if (
        equals_when_enabled($value, $int_or_object, $enabled) && equals_when_enabled($value, $int_or_bool, $enabled)
        || equals_when_enabled($value, $float_or_object, $enabled)
        && equals_when_enabled($value, $float_or_bool, $enabled)
    ) {
        return $value;
    }

    return 0;
}
