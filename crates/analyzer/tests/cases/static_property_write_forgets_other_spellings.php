<?php

declare(strict_types=1);

class Base
{
    public static ?string $value = null;
}

final class Child extends Base
{
    public static function viaStatic(): string
    {
        Base::$value = 'a';
        static::$value = null;

        /**
         * @mago-expect analysis:nullable-return-statement
         * @mago-expect analysis:invalid-return-statement
         */
        return Base::$value;
    }
}

class Holder
{
    protected static ?string $value = null;

    /** @throws RuntimeException */
    public static function get(): string
    {
        if (static::$value === null) {
            throw new RuntimeException('not initialized');
        }

        self::$value = null;

        /**
         * @mago-expect analysis:nullable-return-statement
         * @mago-expect analysis:invalid-return-statement
         */
        return static::$value;
    }

    public static function kept(): string
    {
        self::$value = 'a';

        return self::$value;
    }
}
