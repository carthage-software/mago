<?php

declare(strict_types=1);

function flow_short_ternary(?string $value): string
{
    return $value ?: 'default';
}
