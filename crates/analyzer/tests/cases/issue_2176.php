<?php

declare(strict_types=1);

function issue2176_and(bool $shouldAppend): void
{
    $arr = [];
    $shouldAppend && ($arr[] = 1);

    if ($arr === []) {
        echo 'empty';
    }
}

function issue2176_low_and(bool $shouldAppend): void
{
    $arr = [];
    $shouldAppend and ($arr[] = 1);

    if ($arr === []) {
        echo 'empty';
    }
}

function issue2176_or(bool $shouldSkip): void
{
    $arr = [];
    $shouldSkip || ($arr[] = rand());

    if ($arr === []) {
        echo 'empty';
    }
}

function issue2176_low_or(bool $shouldSkip): void
{
    $arr = [];
    $shouldSkip or ($arr[] = rand());

    if ($arr === []) {
        echo 'empty';
    }
}

function issue2176_chained(bool $first, bool $second): void
{
    $arr = [];
    $first && $second && ($arr[] = 1);

    if ($arr === []) {
        echo 'empty';
    }
}

/**
 * @return list<int>
 */
function issue2176_keeps_assigned_type(bool $shouldAppend): array
{
    $arr = [];
    $shouldAppend && ($arr[] = 1);

    return $arr;
}

function issue2176_reassignment(bool $shouldReplace): void
{
    $value = 'a';
    $shouldReplace && ($value = 'b');

    if ($value === 'a') {
        echo 'unchanged';
    }
}

function issue2176_narrowing_still_applies(?string $left, ?string $right): string
{
    if ($left !== null && $right !== null) {
        return $left . $right;
    }

    return '';
}
