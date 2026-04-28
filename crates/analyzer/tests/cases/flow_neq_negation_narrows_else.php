<?php

declare(strict_types=1);

function flow_neq_negation_narrows_else(null|int $v): int
{
    if ($v === null) {
        $v = 42;
    }

    return $v + 1;
}
