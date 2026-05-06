<?php

declare(strict_types=1);

enum Status: int
{
    case A = 1;
    case B = 2;

    private const MAP = [
        'alpha' => self::A,
        'beta' => self::B,
    ];

    public static function fromString(string $value): ?self
    {
        return array_find(self::MAP, static fn(self $status, string $key) => $key === $value);
    }
}

class Foo
{
    private const X = 1;
    private const Y = 2;
    private const MAP = ['a' => self::X, 'b' => self::Y];

    /** @return array{a: int, b: int} */
    public static function getMap(): array
    {
        return self::MAP;
    }
}
