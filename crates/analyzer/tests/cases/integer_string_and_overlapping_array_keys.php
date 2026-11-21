<?php

declare(strict_types=1);

/**
 * @param non-empty-list<string> $size
 * @param array<int|non-empty-string, string> $values
 * @param array<int, string> $numbers
 */
function integer_string_and_overlapping_array_keys(array $size, array $values, array $numbers, string $name): string
{
    $width = $size['0'];
    /** @mago-expect analysis:possibly-undefined-string-array-index */
    $value = $values[$name];

    /** @mago-expect analysis:mismatched-array-index */
    $number = $numbers['name'];

    /** @mago-expect analysis:mismatched-array-index */
    $other = $size[$name];

    return $width . $value . $number . $other;
}

function superglobal_read_with_string_key(string $name): mixed
{
    return $_GET[$name];
}
