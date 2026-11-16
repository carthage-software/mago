<?php

declare(strict_types=1);

class Named
{
    public function name(): string
    {
        return 'named';
    }

    public function compare(object $other): string
    {
        if ($other::class === static::class) {
            return $other->name();
        }

        if (static::class !== $other::class) {
            return '';
        }

        return $other->name();
    }
}
