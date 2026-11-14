<?php

declare(strict_types=1);

/**
 * @template T of array{as: int}|array{of: int}
 * @param T $shape
 */
function isset_on_template_union_shape(array $shape): int
{
    if (isset($shape['as'])) {
        return $shape['as'];
    }

    return 0;
}
