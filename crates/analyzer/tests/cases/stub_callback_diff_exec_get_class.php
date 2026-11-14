<?php

declare(strict_types=1);

/**
 * @param array<string, int> $a
 * @param array<string, int> $b
 *
 * @return array<string, int>
 */
function stub_udiff_keeps_first(array $a, array $b): array
{
    return array_udiff($a, $b, static fn(int $x, int $y): int => $x <=> $y);
}

/**
 * @return array<array-key, string>
 */
function stub_exec_keeps_input_keys(): array
{
    $out = ['a' => 'x'];
    exec('ls', $out);

    return $out;
}

final class StubGetClassFoo
{
    public function hi(): string
    {
        return 'hi';
    }
}

/**
 * @template T of object
 *
 * @param class-string<T> $class
 *
 * @return T|null
 */
function stub_get_class_find(string $class): ?object
{
    return null;
}

function stub_get_class_binds(StubGetClassFoo $foo): ?string
{
    return stub_get_class_find(get_class($foo))?->hi();
}
