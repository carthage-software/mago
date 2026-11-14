<?php

declare(strict_types=1);

class Base
{
    public const VALUE = true;

    public const KIND = 1;

    public function enabled(): string
    {
        if (static::VALUE) {
            return 'enabled';
        }

        return 'disabled';
    }

    public function kind(): string
    {
        return $this::KIND === 2 ? 'two' : 'other';
    }

    public function exact(): string
    {
        // @mago-expect analysis:redundant-condition
        if (self::VALUE) {
            return 'enabled';
        }

        return 'disabled';
    }
}

final class Child extends Base
{
    public const VALUE = false;

    public const KIND = 2;
}

function through_instance(Base $base): string
{
    return $base::VALUE ? 'enabled' : 'disabled';
}
