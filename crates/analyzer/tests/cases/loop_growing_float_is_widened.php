<?php

declare(strict_types=1);

function loop_growing_float(): string
{
    $total = 0.0;
    while (rand(0, 1) === 1) {
        $total += 1.5;
    }

    if ($total === 6.0) {
        return 'six';
    }

    return 'other';
}
