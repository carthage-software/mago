<?php

declare(strict_types=1);

function flow_is_scalar_narrow(mixed $v): string
{
    if (is_scalar($v)) {
        return (string) $v;
    }

    return '';
}
