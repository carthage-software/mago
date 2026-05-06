<?php

declare(strict_types=1);

function flow_empty_string(?string $value): int
{
    if (!empty($value)) {
        return strlen($value);
    }

    return 0;
}
