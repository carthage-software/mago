<?php

declare(strict_types=1);

final class Label
{
    public function __toString(): string
    {
        return 'label';
    }
}

/**
 * @param non-empty-list<array{id: stdClass, n: int}> $rows
 */
function object_index(array $rows): void
{
    /**
     * @mago-expect analysis:invalid-argument
     * @mago-expect analysis:impossible-assignment
     */
    $result = array_column($rows, 'n', 'id');
}

/**
 * @param non-empty-list<array{0: Label, 1: int}> $rows
 */
function stringable_index(array $rows): void
{
    /**
     * @mago-expect analysis:invalid-argument
     * @mago-expect analysis:impossible-assignment
     */
    $result = array_column($rows, 1, 0);
}

/**
 * @param list<array{id: int|stdClass, n: int}> $rows
 * @return array<int, int>
 */
function maybe_object_index(array $rows): array
{
    /** @mago-expect analysis:possibly-invalid-argument */
    return array_column($rows, 'n', 'id');
}

/**
 * @param list<array{id: int, n: int}> $rows
 * @return array<int, int>
 */
function int_index(array $rows): array
{
    return array_column($rows, 'n', 'id');
}
