<?php

declare(strict_types=1);

namespace App;

use function assert;
use function array_all;
use function array_filter;
use function is_int;

/**
 * @return array<int>
 */
function filtered(array $a): array
{
    return array_filter($a, static fn($id) => is_int($id));
}

/**
 * @return array<int>
 */
function asserted(array $a): array
{
    assert(array_all($a, static fn($id) => is_int($id)), description: 'all ints');

    return $a;
}
