<?php

declare(strict_types=1);

function flow_narrow_then_mutate(null|int $v): int
{
    if ($v === null) {
        return -1;
    }

    $v = $v * 2;

    return $v;
}
