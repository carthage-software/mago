<?php

declare(strict_types=1);

/**
 * @param array<string, int> $input
 * @return array<string, int>
 */
function reverse_string_keyed(array $input): array
{
    return array_reverse($input);
}

/**
 * @param array<string, int> $input
 * @return array<string, int>
 */
function reverse_string_keyed_preserve(array $input): array
{
    return array_reverse($input, true);
}

/**
 * @param list<string> $input
 * @return list<string>
 */
function reverse_list(array $input): array
{
    return array_reverse($input);
}
