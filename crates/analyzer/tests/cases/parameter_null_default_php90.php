<?php

declare(strict_types=1);

// On PHP >= 9.0 a native type no longer becomes nullable through a `null` default.
// @mago-expect analysis:invalid-parameter-default-value
function native_null_default_php90(string $value = null): string
{
    return $value;
}

/**
 * An untyped parameter can still default to `null`, so the docblock type gets the `null`.
 *
 * @param string $value
 */
function docblock_null_default_php90($value = null): string
{
    return $value ?? 'none';
}
