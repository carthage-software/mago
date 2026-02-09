<?php

declare(strict_types=1);

function foo(string $type): string
{
    switch ($type) {
        case 'a':
            return 'yes';
        case 'b':
            if (rand(min: 0, max: 1) === 1) {
                return 'yes';
            }
        case 'c':
            return 'no';
        default:
            return 'unknown';
    }
}

foo('b');
