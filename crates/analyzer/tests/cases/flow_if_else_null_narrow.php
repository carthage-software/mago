<?php

declare(strict_types=1);

function take_string(string $s): void
{
    echo $s;
}

function flow_if_else_null_narrow(null|string $value): void
{
    if ($value === null) {
        $value = 'default';
    } else {
        echo strlen($value);
    }

    take_string($value);
}
