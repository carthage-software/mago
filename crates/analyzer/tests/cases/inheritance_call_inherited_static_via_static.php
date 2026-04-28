<?php

declare(strict_types=1);

class InhStaticInhParent
{
    public static function value(): int
    {
        return 1;
    }
}

class InhStaticInhChild extends InhStaticInhParent
{
    public static function double(): int
    {
        return static::value() * 2;
    }
}

echo InhStaticInhChild::double();
