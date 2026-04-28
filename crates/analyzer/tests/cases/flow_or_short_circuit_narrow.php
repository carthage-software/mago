<?php

declare(strict_types=1);

function flow_or_short_circuit_narrow(null|string $v): string
{
    if ($v === null || $v === '') {
        return 'default';
    }

    return $v;
}
