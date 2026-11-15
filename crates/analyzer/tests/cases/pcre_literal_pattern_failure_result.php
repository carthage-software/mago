<?php

declare(strict_types=1);

function replace_with_literal_pattern(string $subject): string
{
    return trim(preg_replace('/[?.]$/', 'X', $subject));
}

function replace_with_literal_patterns(string $subject): string
{
    $result = preg_replace(['/a/', '#b#i'], ['x', 'y'], $subject);

    return trim($result);
}

function replace_with_callback(string $subject): string
{
    return preg_replace_callback('/\d+/', static fn(array $matches): string => implode('', $matches) . '!', $subject);
}

function result_can_still_be_checked(string $subject): string
{
    $result = preg_replace('/x/u', '', $subject);
    if (null === $result) {
        return '';
    }

    return $result;
}

function match_with_literal_pattern(string $subject): int
{
    return preg_match('/a/', $subject);
}

/** @return list<string> */
function split_with_literal_pattern(string $subject): array
{
    return preg_split('/,\s*/', $subject);
}

function replace_with_dynamic_pattern(string $pattern, string $subject): string
{
    // @mago-expect analysis:possibly-null-argument
    return trim(preg_replace($pattern, 'X', $subject));
}

function replace_with_unknown_modifier(string $subject): string
{
    // @mago-expect analysis:possibly-null-argument
    return trim(preg_replace('/a/k', 'X', $subject));
}

function match_with_dynamic_pattern(string $pattern, string $subject): int
{
    // @mago-expect analysis:falsable-return-statement,invalid-return-statement
    return preg_match($pattern, $subject);
}
