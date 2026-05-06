<?php

declare(strict_types=1);

function flow_isset_variable(?string $value): string
{
    if (isset($value)) {
        return $value;
    }

    return 'none';
}
