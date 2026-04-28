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
    /** @mago-expect analysis:incompatible-parameter-type */
    public static function build(string $x): int
    {
        return strlen($x);
    }
}
