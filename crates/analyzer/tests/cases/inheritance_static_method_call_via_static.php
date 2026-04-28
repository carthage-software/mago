<?php

declare(strict_types=1);

class InhStaticVia
{
    public static function value(): int
    {
        return 1;
    }

    public static function compute(): int
    {
        return static::value() + 1;
    }
}

class InhStaticViaChild extends InhStaticVia
{
    #[\Override]
    public static function value(): int
    {
        return 10;
    }
}

echo InhStaticViaChild::compute();
