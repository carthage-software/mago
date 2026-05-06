<?php

declare(strict_types=1);

class InhStaticSigParent
{
    public static function build(int $x): int
    {
        return $x;
    }
}

class InhStaticSigChild extends InhStaticSigParent
{
    public static function build(string $x): int
    {
        return strlen($x);
    }
}
