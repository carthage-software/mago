<?php

declare(strict_types=1);

function pms_callback_wide(): string|false // @mago-expect analysis:overly-wide-return-type
{
    return 'test';
}

class Bar
{
}

class BarHolder
{
    public function __construct(
        public Bar $bar,
    ) {}

    public function getBar(): ?Bar // @mago-expect analysis:overly-wide-return-type
    {
        return $this->bar;
    }
}

function scalar_union_drops_array(): int|string|array // @mago-expect analysis:overly-wide-return-type
{
    if (mt_rand(0, 1)) {
        return 1;
    }

    return 'a';
}

function int_or_false_always_int(): int|false // @mago-expect analysis:overly-wide-return-type
{
    if (mt_rand(0, 1)) {
        return 1;
    }

    return 2;
}

class Widget
{
    public function find(int $id): ?Widget // @mago-expect analysis:overly-wide-return-type
    {
        return new Widget();
    }
}

$wide_closure =
    static function (): int|false { // @mago-expect analysis:overly-wide-return-type
        return 42;
    };

function exact_match(): int
{
    return 1;
}

function exact_union(bool $b): int|string
{
    if ($b) {
        return 1;
    }

    return 'a';
}

function uses_all_branches(int $x): int|string|null
{
    if ($x > 0) {
        return 'positive';
    }

    if ($x < 0) {
        return -1;
    }

    return null;
}

function returns_literal_string(): string
{
    return 'hello';
}

function returns_literal_int(): int
{
    return 42;
}

function returns_mixed(): mixed
{
    return 1;
}

function returns_void(): void
{
    echo 'side effect';
}

/**
 * @return Generator<int, int, mixed, void>
 */
function generator_with_wide_return(): Generator
{
    yield 1;
}

interface LookupContract
{
    public function find(int $id): ?Widget;
}

class AlwaysFoundLookup implements LookupContract
{
    public function find(int $id): ?Widget
    {
        return new Widget();
    }
}
