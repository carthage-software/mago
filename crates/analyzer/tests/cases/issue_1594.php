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
        // @mago-expect analysis:possibly-undefined-string-array-index - `image_id` might not exist
        // @mago-expect analysis:possibly-null-array-index - even if it does, it could be null
        if (!isset($stars[$row['image_id']])) {
            // @mago-expect analysis:possibly-undefined-string-array-index - again, might not exist
            $stars[(int) $row['image_id']] = 0;
        }

        // @mago-expect analysis:possibly-undefined-string-array-index - image_id might not exist
        // @mago-expect analysis:possibly-null-argument - again, even if it does, might be null
        if (!array_key_exists($row['image_id'], $stars)) {
            // @mago-expect analysis:possibly-undefined-string-array-index - might not exist, again
            $stars[(int) $row['image_id']] = 0;
        }

        // @mago-expect analysis:possibly-undefined-string-array-index - image_id might not exist
        // @mago-expect analysis:possibly-undefined-string-array-index - vote might not exist
        $stars[(int) $row['image_id']] += (int) $row['vote'];
    }
}

/** @param array<string, string|null> $row */
function test_null_coalesce_func_call(array $row): void
{
    // @mago-expect analysis:possibly-undefined-string-array-index - missing might not exist
    echo i_might_return_null($row['missing']) ?? 'default';
}

/** @param array<string, string>|false $row */
function test_that(array|false $row): void
{
    // @mago-expect analysis:possibly-false-array-access - $row can be false
    // @mago-expect analysis:possibly-undefined-string-array-index - if not, the key might not exist
    // @mago-expect analysis:possibly-null-argument - if $row is false, arg ends up `null`
    echo i_might_return_null_too($row['missing']) ?? 'default';
}

/** @param array<string, string>|null $row */
function test_that2(?array $row): void
{
    // @mago-expect analysis:possibly-null-array-access - $row can be null
    // @mago-expect analysis:possibly-undefined-string-array-index - if not, the key might not exist
    // @mago-expect analysis:possibly-null-argument - if $row is null, arg ends up `null`
    echo i_might_return_null_too($row['missing']) ?? 'default';
}
