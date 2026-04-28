<?php

declare(strict_types=1);

function flow_is_null_narrow(null|string $value): string
{
    if (is_null($value)) {
        return 'null';
    }

    return $value;
}
