<?php

declare(strict_types=1);

namespace Issue2309;

final readonly class Foo
{
    private const array BITS = [
        0b0001,
        0b0010,
        0b0100,
        0b1000,
    ];

    /** @param int-mask-of<self::BITS> $value */
    public function __construct(
        public int $value,
    ) {}

    /** @return list<value-of<self::BITS>> */
    public function splitBits(): array
    {
        $bits = [];
        foreach (self::BITS as $bit) {
            if (($bit & $this->value) !== $bit) {
                continue;
            }

            $bits[] = $bit;
        }

        return $bits;
    }
}

/**
 * @param 1|2|4|8 $value
 * @return 1|2|4|8
 */
function preserveLiteralUnion(int $value, int $other): int
{
    if ($other === $value) {
        return $value;
    }

    return 1;
}

/**
 * @param int<1, 10> $value
 * @param int<5, 15> $other
 * @return int<5, 10>
 */
function intersectIntegerRanges(int $value, int $other): int
{
    if ($value === $other) {
        return $value;
    }

    return 5;
}
