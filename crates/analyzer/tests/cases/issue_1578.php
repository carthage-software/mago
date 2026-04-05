<?php

declare(strict_types=1);

function unlikely_fatal_error(string $_msg): never
{
    exit(1);
}

function test(int $date_in, string $range): int
{
    switch ($range) {
        case 'm':
            $s = date('Y-m-01 00:00:00', $date_in);
            $tmp_start = date_create($s);

            break;
        default:
            unlikely_fatal_error('illegal range');
    }

    if ($tmp_start === false) {
        return 0;
    }

    return (int) $tmp_start->format('U');
}

function test_if_style(int $code): string
{
    if ($code === 1) {
        $message = 'one';
    } elseif ($code === 2) {
        $message = 'two';
    } else {
        unlikely_fatal_error('invalid code');
    }

    return $message;
}

// Nested: a never-returning call inside a block statement.
function test_block(int $n): int
{
    switch ($n) {
        case 1:
            $x = 10;
            break;
        default:
            {
                unlikely_fatal_error('bad');
            }
    }

    return $x;
}
