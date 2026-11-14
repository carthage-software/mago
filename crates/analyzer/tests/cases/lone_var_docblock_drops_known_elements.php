<?php

declare(strict_types=1);

/**
 * @param array{i: int|null} $a
 */
function redeclared(array $a): int
{
    if ($a['i'] !== null) {
        return 0;
    }

    /** @var array{i: int} $a */;

    return $a['i'];
}
