<?php

declare(strict_types=1);

function flow_ternary_narrow(null|string $value): string
{
    return $value === null ? 'default' : $value;
}
