<?php

declare(strict_types=1);

final class D
{
    /** @var array{0: string, 1: string} */
    private const VALUES = [0 => 'one', 1 => 'two'];

    public string $name;

    /**
     * @param non-negative-int $key
     */
    private function __construct(
        public int $key,
    ) {
        $this->name = self::VALUES[$key] ?? 'unknown';
    }
}

/**
 * @param list<string> $values
 * @param non-negative-int $key
 */
function test_list_access(array $values, int $key): string
{
    return $values[$key] ?? 'default';
}

/**
 * @param array{name: string, age?: int} $data
 */
function test_shape_access(array $data, string $key): mixed
{
    return $data[$key] ?? null;
}

/**
 * @param list{string, string, string} $items
 * @param 0|1|2 $index
 */
function test_list_with_known_elements_literal_union(array $items, int $index): string
{
    return $items[$index] ?? 'fallback';
}

/**
 * @param list{string, string, string} $items
 */
function test_list_with_known_elements_literal_union2(array $items, int $index): string
{
    return $items[$index] ?? 'fallback';
}
