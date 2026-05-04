<?php

declare(strict_types=1);

/**
 * @phpstan-type Valid array{valid: true, result: string}
 * @phpstan-type Invalid array{valid: false, errorCode: string}
 */
class Types
{
}

/** @param !Types::Valid $_ */
function use_valid(array $_): void
{
}

/** @param !Types::Invalid $_ */
function use_invalid(array $_): void
{
}

/**
 * @param array{valid: true, result: string}|array{valid: false, errorCode: string} $input
 */
function test_with_array_shapes(array $input): void
{
    if ($input['valid']) {
        use_valid($input);
    } else {
        use_invalid($input);
    }
}

/**
 * @param !Types::Valid|!Types::Invalid $input
 */
function test_with_type_aliases(array $input): void
{
    if ($input['valid']) {
        use_valid($input);
    } else {
        use_invalid($input);
    }
}
