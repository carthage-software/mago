<?php

declare(strict_types=1);

function flow_low_prec_or(?string $a, ?string $b): string
{
    if ($a === null or $b === null) {
        return 'missing';
    }

    return $a . $b;
}
