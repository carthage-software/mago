<?php

declare(strict_types=1);

function flow_null_coalesce(?string $value): string
{
    return $value ?? 'default';
}
