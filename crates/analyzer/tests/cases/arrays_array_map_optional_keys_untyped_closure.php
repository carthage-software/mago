<?php

declare(strict_types=1);

/**
 * `array_map` must preserve the sealed shape rather than widen it to include
 * a `list<string>` that would clash with the declared return type.
 *
 * @param array{foo?: string} $x
 * @return array{foo?: string}
 */
function map_optional_shape(array $x): array
{
    return array_map(fn($value) => $value, $x);
}
