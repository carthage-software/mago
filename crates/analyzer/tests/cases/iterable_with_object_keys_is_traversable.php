<?php

declare(strict_types=1);

/**
 * @param list<int>|Traversable<object, int> $values
 */
function accept_object_keys($values): void
{
}

/**
 * @param array<string, int>|Traversable<string|object, int> $values
 */
function accept_string_or_object_keys($values): void
{
}

/**
 * @param iterable<object, int> $values
 */
function pass_object_keys(iterable $values): void
{
    accept_object_keys($values);
}

/**
 * @param iterable<string|object, int> $values
 */
function pass_string_or_object_keys(iterable $values): void
{
    accept_string_or_object_keys($values);
}
