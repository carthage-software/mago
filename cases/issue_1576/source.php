<?php

declare(strict_types=1);

function test(): string
{
    $c = mt_rand(-50, max: 50);
    if (0 == $c) {
        return '';
    }

    if (0 == $c) {
        $ti = 'leer';
    }

    return 'some';
}

function test_swapped(): string
{
    $c = mt_rand(-50, max: 50);
    if ($c == 0) {
        return '';
    }

    if ($c == 0) {
        $ti = 'leer';
    }

    return 'some';
}

function test_not_equal(): string
{
    $c = mt_rand(-50, max: 50);
    if ($c == 0) {
        return '';
    }

    if ($c != 0) {
        return 'nonzero';
    }

    return 'some';
}

function test_not_equal_ang(): string
{
    $c = mt_rand(-50, max: 50);
    if ($c == 0) {
        return '';
    }

    if ($c <> 0) {
        return 'nonzero';
    }

    return 'some';
}

function test_mixed_primitives_not_flagged(int $i, string $s, float $f): void
{
    if ($i == $s) {
        echo 'maybe equal via juggling';
    }
    if ($i == $f) {
        echo 'maybe equal via numeric coercion';
    }
}
