<?php

declare(strict_types=1);

function flow_negated_is_int(int|string $value): int
{
    if (!is_int($value)) {
        return strlen($value);
    }

    return $value + 1;
}
