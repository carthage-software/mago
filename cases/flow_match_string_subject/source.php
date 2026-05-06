<?php

declare(strict_types=1);

function flow_match_string_subject(string $s): int
{
    return match ($s) {
        'a' => 1,
        'b' => 2,
        default => 0,
    };
}
