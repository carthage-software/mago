<?php

declare(strict_types=1);

/**
 * @param int<-5, 5> $v
 */
function flow_int_eq_negative(int $v): string
{
    if ($v === -1) {
        return 'minus-one';
    }

    return 'other';
}
