<?php

declare(strict_types=1);

function flow_and_short_circuit_narrow(null|string $v): int
{
    if ($v !== null && strlen($v) > 0) {
        return strlen($v);
    }

    return 0;
}
