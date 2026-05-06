<?php

declare(strict_types=1);

function flow_ternary_narrow(?string $value): string
{
    return $value === null ? 'default' : $value;
}
