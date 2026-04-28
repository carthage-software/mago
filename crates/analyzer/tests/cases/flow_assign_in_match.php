<?php

declare(strict_types=1);

function flow_assign_in_match(int $code): string
{
    $msg = match ($code) {
        200 => 'ok',
        404 => 'not found',
        default => 'other',
    };

    return $msg;
}
