<?php

declare(strict_types=1);

function flow_null_coalesce(null|string $value): string
{
    return $value ?? 'default';
}
