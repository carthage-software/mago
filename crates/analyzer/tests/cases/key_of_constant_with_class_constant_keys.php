<?php

declare(strict_types=1);

final class Foo
{
    public const BAR_A = 'a';
    public const BAR_B = 'b';

    /** @var array<self::BAR_*, int> */
    public const BARS = [self::BAR_A => 1, self::BAR_B => 2];

    /** @var key-of<self::BARS> */
    // @mago-expect analysis:invalid-constant-value
    public const BAD = 'zzz';

    /** @param key-of<self::BARS> $key */
    public static function pick(string $key): int
    {
        return self::BARS[$key];
    }
}

echo Foo::pick('a');
