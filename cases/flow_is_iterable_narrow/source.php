<?php

declare(strict_types=1);

function flow_is_iterable_narrow(mixed $v): int
{
    if (is_iterable($v)) {
        $count = 0;
        foreach ($v as $_item) {
            $count++;
        }
        return $count;
    }

    return 0;
}
