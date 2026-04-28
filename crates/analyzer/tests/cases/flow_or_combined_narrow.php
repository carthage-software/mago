<?php

declare(strict_types=1);

function flow_or_combined_narrow(null|int|string $v): string
{
    if (is_int($v) || is_string($v)) {
        return (string) $v;
    }

    return 'null';
}
