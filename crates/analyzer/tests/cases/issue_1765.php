<?php

declare(strict_types=1);

namespace Foo;

function add_three(array $x): array
{
    $doit = (bool) random_int(0, 1);

    /** @mago-expect analysis:invalid-operand */
    if ($x && $doit) {
        $x[] = 3;
    }

    if ($x) {
        $x[] = 3;
    }

    return $x;
}
