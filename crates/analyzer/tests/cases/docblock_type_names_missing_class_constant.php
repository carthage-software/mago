<?php

declare(strict_types=1);

enum Suit: string
{
    case Hearts = 'h';
}

interface HasLimit
{
    public const LIMIT = 1;
}

final class Foo implements HasLimit
{
    public const BAR_A = 'a';

    // @mago-expect analysis:non-existent-class-constant
    /** @var self::MISSING */
    public const ONE = 'z';

    // @mago-expect analysis:non-existent-class-constant
    /** @var self::NOPE_* */
    public const TWO = 'z';

    // @mago-expect analysis:non-existent-class-constant
    /** @param self::MISSING $a */
    public static function take(mixed $a): void
    {
        echo gettype($a);
    }

    // @mago-expect analysis:non-existent-class-like
    /** @param Nope::X $b */
    public static function takeUnknown(mixed $b): void
    {
        echo gettype($b);
    }

    /**
     * @param self::BAR_*|self::LIMIT|Suit::Hearts|Suit::*|self::class $value
     */
    public static function fine(mixed $value): void
    {
        echo gettype($value);
    }
}
