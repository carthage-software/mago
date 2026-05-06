<?php

declare(strict_types=1);

function flow_is_bool_narrow(bool|int $value): int
{
    if (is_bool($value)) {
        return $value ? 1 : 0;
    }

    return $value;
}
