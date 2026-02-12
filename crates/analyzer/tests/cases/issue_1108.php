<?php declare(strict_types=1);

/**
 * @param non-empty-list<string> $parts
 * @return list<null|string>
 */
function spread_unknown_count_list_with_extra_values(array $parts): array
{
    return [...$parts, null, null];
}

/**
 * @return list<null|string>
 */
function spread_explode_result_with_extra_values(): array
{
    $parts = explode('/', 'a/b/c');
    return [...$parts, null, null];
}

/**
 * @param non-empty-list<string> $parts
 * @return non-empty-list<int|null|string>
 */
function value_before_spread_and_after(array $parts): array
{
    return [1, ...$parts, null];
}
