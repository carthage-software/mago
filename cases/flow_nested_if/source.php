<?php

declare(strict_types=1);

function flow_nested_if(?int $a, ?int $b): int
{
    if ($a !== null) {
        if ($b !== null) {
            return $a + $b;
        }
        return $a;
    }

    return 0;
}
