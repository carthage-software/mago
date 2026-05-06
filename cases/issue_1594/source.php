<?php

declare(strict_types=1);

function i_might_return_null(?string $x): ?string
{
    return $x;
}

function i_might_return_null_too(string $x): ?string
{
    return rand(0, 1) ? $x : null;
}

/** @return false|array<string,string|null> */
function fassoc(): false|array
{
    if (mt_rand(0, max: 5) === 0) {
        return false;
    }
    return ['image_id' => (string) mt_rand(1, max: 20)];
}

function test(): void
{
    $stars = [];
    while ($row = FASSOC()) {
        if (!isset($stars[$row['image_id']])) {
            $stars[(int) $row['image_id']] = 0;
        }

        if (!array_key_exists($row['image_id'], $stars)) {
            $stars[(int) $row['image_id']] = 0;
        }

        $stars[(int) $row['image_id']] += (int) $row['vote'];
    }
}

/** @param array<string, string|null> $row */
function test_null_coalesce_func_call(array $row): void
{
    echo i_might_return_null($row['missing']) ?? 'default';
}

/** @param array<string, string>|false $row */
function test_that(array|false $row): void
{
    echo i_might_return_null_too($row['missing']) ?? 'default';
}

/** @param array<string, string>|null $row */
function test_that2(?array $row): void
{
    echo i_might_return_null_too($row['missing']) ?? 'default';
}
