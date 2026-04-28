<?php

declare(strict_types=1);

function flow_low_prec_or(null|string $a, null|string $b): string
{
    if ($a === null or $b === null) {
        return 'missing';
    }

    return $a . $b;
}
