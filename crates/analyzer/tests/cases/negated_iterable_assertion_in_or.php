<?php

declare(strict_types=1);

/**
 * @phpstan-assert-if-true iterable<mixed, string> $value
 */
function all_strings(mixed $value): bool
{
    if (!is_iterable($value)) {
        return false;
    }

    $items = is_array($value) ? $value : iterator_to_array($value);

    return array_all($items, static fn(mixed $item): bool => is_string($item));
}

/**
 * @param array<array-key, mixed> $values
 */
function rejects_mixed_array(array $values): void
{
    if (empty($values) || !all_strings($values)) {
        return;
    }

    echo count($values);
}

/**
 * @param list<mixed> $values
 */
function rejects_mixed_list(array $values, bool $flag): bool
{
    return $flag || !all_strings($values);
}

/**
 * @param Traversable<mixed, mixed> $values
 */
function rejects_mixed_traversable(Traversable $values, bool $flag): void
{
    if ($flag || !all_strings($values)) {
        return;
    }

    echo 'all strings';
}

/**
 * @param ArrayObject<array-key, mixed> $values
 */
function rejects_mixed_array_object(ArrayObject $values, bool $flag): void
{
    if ($flag || !all_strings($values)) {
        return;
    }

    echo 'all strings';
}
