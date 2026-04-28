<?php

declare(strict_types=1);

function flow_match_with_default(int $code): string
{
    return match ($code) {
        200 => 'ok',
        404 => 'not found',
        default => 'other',
    };
}
