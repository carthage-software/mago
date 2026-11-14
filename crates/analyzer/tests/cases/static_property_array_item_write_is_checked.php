<?php

declare(strict_types=1);

final class Registry
{
    /** @var array<string, int> */
    public static array $map = [];

    /** @var list<int> */
    public static array $list = [];

    public static function valid(): void
    {
        self::$map['a'] = 1;
        self::$list[] = 2;
    }

    public static function invalid(): void
    {
        /** @mago-expect analysis:invalid-property-assignment-value */
        self::$map['a'] = 'str';
        /** @mago-expect analysis:invalid-property-assignment-value */
        self::$list[] = 'str';
        /** @mago-expect analysis:invalid-property-assignment-value */
        preg_match('/x/', 'x', self::$map['b']);
    }
}
