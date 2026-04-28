<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:possibly-null-operand
 * @mago-expect analysis:null-operand
 */
function flow_loose_eq_null(null|int $v): int
{
    if ($v == null) {
        return -1;
    }

    return $v + 1;
}
