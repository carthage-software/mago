<?php

declare(strict_types=1);

function img_meta_get_lens(string $t): string
{
    $garbage = match (true) {
        'None' === $t => true,
        'n/a' === $t => true,
        str_contains($t, 'Binary data') => true,
        true => false,
    };

    if ($garbage) {
        return '';
    }

    return $t;
}
