<?php

declare(strict_types=1);

function flow_unreachable_after_return(int $v): int
{
    if ($v > 0) {
        return $v;
    }

    return -$v;
}
