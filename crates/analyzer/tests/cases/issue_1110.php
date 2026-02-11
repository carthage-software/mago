<?php declare(strict_types=1);

class Test
{
    public const ONE = 'one';
    public const TWO = 'two';

    /** @var array<string, array{one?: int, two: int}> */
    public const DATA = [
        'key1' => [
            self::ONE => null,
            self::TWO => 30 * 60,
        ],
        'key2' => [
            self::ONE => 9 * 3_600,
            self::TWO => 15 * 60,
        ],
    ];

    public function test(string $key): int
    {
        if (!array_key_exists($key, self::DATA)) {
            return 0;
        }

        return self::DATA[$key][self::TWO];
    }
}
