<?php

declare(strict_types=1);

function greet(string $name): string
{
    return "Hello {$name}";
}

class Base
{
    public const ?string NAME = null;

    public static ?string $value = null;

    public static function late(): string
    {
        if (static::NAME !== null) {
            return greet(static::NAME);
        }

        return 'none';
    }

    public static function exact(): string
    {
        // @mago-expect analysis:redundant-comparison
        // @mago-expect analysis:impossible-condition
        if (self::NAME !== null) {
            // @mago-expect analysis:no-value
            return greet(self::NAME);
        }

        return 'none';
    }

    public static function separate(): string
    {
        if (self::$value === null) {
            return 'none';
        }

        // @mago-expect analysis:possibly-null-argument
        return greet(static::$value);
    }

    public static function same_class(): string
    {
        return self::class === static::class ? 'base' : 'child';
    }
}

final class Child extends Base
{
    public const ?string NAME = 'child';
}
