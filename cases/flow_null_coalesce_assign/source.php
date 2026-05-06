<?php

declare(strict_types=1);

function flow_null_coalesce_assign(?string $value): string
{
    $value ??= 'default';

    return $value;
}
