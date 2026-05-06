<?php

declare(strict_types=1);

function flow_low_prec_and(?string $a, ?string $b): string
{
    if ($a !== null and $b !== null) {
        return $a . $b;
    }

    return '';
}
