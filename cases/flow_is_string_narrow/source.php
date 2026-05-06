<?php

declare(strict_types=1);

function flow_is_string_narrow(int|string $value): string
{
    if (is_string($value)) {
        return $value;
    }

    return (string) $value;
}
