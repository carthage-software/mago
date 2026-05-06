<?php

enum Color: string
{
    case Red = 'red';
    case Green = 'green';
    case Blue = 'blue';

    /** @return callable(Color): string */
    public static function getHexCallback(): callable
    {
        return static fn(self $color): string => match ($color) {
            self::Red => '#FF0000',
            self::Green => '#00FF00',
            self::Blue => '#0000FF',
        };
    }

    /** @return list<string> */
    public static function getAllAsHex(): array
    {
        return array_map(self::getHexCallback(), self::cases());
    }
}
