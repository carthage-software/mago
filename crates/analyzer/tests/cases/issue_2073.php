<?php

declare(strict_types=1);

/** @param array{a?: string} $a */
function f(array $a): void
{
    if (array_is_list($a)) {
        echo "reached at runtime when \$a === []\n";
    }
}

f([]);
