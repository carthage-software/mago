<?php

enum Foo: string
{
    case A = 'a';
    case B = 'b';
    case C = 'c';
    case D = 'd';

    public function toResult(): string
    {
        return match ($this) {
            // @mago-format-ignore-next
            self::A, self::B => 'ab',
            self::C, self::D => 'cd',
        };
    }
}
