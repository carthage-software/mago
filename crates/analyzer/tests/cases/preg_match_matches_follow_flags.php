<?php

declare(strict_types=1);

/**
 * @param array<array{string, int<-1, max>}> $matches
 */
function take_offsets(array $matches): void
{
}

/**
 * @param array<list<string>> $matches
 */
function take_pattern_order(array $matches): void
{
}

/**
 * @param list<array<string>> $matches
 */
function take_set_order(array $matches): void
{
}

function unmatched_as_null(string $subject): int
{
    $matches = [];
    preg_match('/^(a)?(b)$/', $subject, $matches, PREG_UNMATCHED_AS_NULL);

    $unmatched = 0;
    foreach ($matches as $match) {
        if ($match === null) {
            $unmatched++;
        }
    }

    return $unmatched;
}

function offset_capture(string $subject): void
{
    $matches = [];
    preg_match('/a/', $subject, $matches, PREG_OFFSET_CAPTURE);

    take_offsets($matches);
}

function default_flags(string $subject): string
{
    $matches = [];
    preg_match('/a/', $subject, $matches);

    return implode(',', $matches);
}

function all_pattern_order(string $subject): void
{
    $matches = [];
    preg_match_all('/a/', $subject, $matches);

    take_pattern_order($matches);
}

function all_set_order(string $subject): void
{
    $matches = [];
    preg_match_all('/a/', $subject, $matches, PREG_SET_ORDER);

    take_set_order($matches);
}
