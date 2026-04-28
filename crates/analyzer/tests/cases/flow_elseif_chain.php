<?php

declare(strict_types=1);

function flow_elseif_chain(int|string|float $v): string
{
    if (is_int($v)) {
        return 'i:' . $v;
    } elseif (is_float($v)) {
        return 'f';
    } else {
        return $v;
    }
}
