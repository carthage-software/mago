<?php

declare(strict_types=1);

function flow_is_numeric_narrow(mixed $v): float
{
    if (is_numeric($v)) {
        return (float) $v;
    }

    return 0.0;
}
