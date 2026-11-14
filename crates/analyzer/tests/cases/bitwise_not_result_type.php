<?php

declare(strict_types=1);

const FLAG_A = 1;

function clear_flag(int $flags): int
{
    $cleared = $flags & ~FLAG_A;

    if ($cleared === 6) {
        return 6;
    }

    return $cleared;
}

function negative_six(): int
{
    return ~5;
}

function from_float(float $value): int
{
    return ~$value;
}

function from_string(string $value): string
{
    return ~$value;
}

function always_true(): void
{
    // @mago-expect analysis:redundant-comparison
    // @mago-expect analysis:redundant-condition
    if (~5 === -6) {
        echo 'minus six';
    }
}
