<?php

declare(strict_types=1);

function flow_low_prec_and(null|string $a, null|string $b): string
{
    if ($a !== null and $b !== null) {
        return $a . $b;
    }

    return '';
}
