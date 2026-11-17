<?php

declare(strict_types=1);

/**
 * @param non-empty-list<string> $parts
 */
function take_non_empty_parts(array $parts): void
{
}

/**
 * @param int<-1, 1> $limit
 */
function explode_limits(string $s, int $limit): void
{
    take_non_empty_parts(explode(' ', $s));
    take_non_empty_parts(explode(' ', $s, 2));
    take_non_empty_parts(explode(' ', $s, 0));

    /** @mago-expect analysis:possibly-invalid-argument */
    take_non_empty_parts(explode(' ', $s, $limit));
}

function explode_first(string $s): string
{
    return explode(' ', $s, -2)[0] ?? '';
}

/**
 * @return non-empty-list<string>
 */
function explode_negative(string $s): array
{
    /** @mago-expect analysis:invalid-return-statement */
    return explode('-', $s, -1);
}
