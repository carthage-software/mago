<?php

declare(strict_types=1);

class Foo
{
    /** @var array<int, array{user: string}> */
    public const array DEFAULTS = [
        ['user' => 'a'],
    ];

    /** @var array<int, array{user: string}> */
    public readonly array $data;

    /** @param array<int, array{user: string}> $data */
    public function __construct(array $data = [])
    {
        $this->data = array_replace(self::DEFAULTS, $data);
    }
}

/**
 * @param array<string, int> $defaults
 * @param array<string, int> $overrides
 * @return array<string, int>
 */
function test_array_replace(array $defaults, array $overrides): array
{
    return array_replace($defaults, $overrides);
}

/**
 * @param array<string, int> $defaults
 * @param array<string, int> $overrides
 * @return array<string, int>
 */
function test_array_replace_recursive(array $defaults, array $overrides): array
{
    return array_replace_recursive($defaults, $overrides);
}

/**
 * @param array<int, array{user: string}> $defaults
 * @param array<int, array{user: string}> $data
 * @return array<int, array{user: string}>
 */
function test_shaped_values(array $defaults, array $data): array
{
    return array_replace($defaults, $data);
}
