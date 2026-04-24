<?php

declare(strict_types=1);

/**
 * @param mixed[] $q
 *
 * @mago-expect analysis:invalid-argument
 */
function strict_builtin_too_few_params(array $q): bool
{
    return array_all($q, is_scalar(...));
}

/**
 * @param mixed[] $q
 */
function user_defined_callback_is_fine(array $q): bool
{
    return array_all($q, function (mixed $value): bool {
        return is_scalar($value);
    });
}

/**
 * @param mixed[] $q
 */
function arrow_callback_is_fine(array $q): bool
{
    return array_all($q, fn(mixed $value): bool => is_scalar($value));
}

/**
 * @param mixed[] $q
 */
function explicit_two_param_callback(array $q): bool
{
    return array_all($q, fn(mixed $value, int|string $key): bool => is_scalar($value));
}
