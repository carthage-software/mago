<?php

declare(strict_types=1);

function flow_early_return_narrow(?string $value): int
{
    if ($value === null) {
        return 0;
    }

    return strlen($value);
}
