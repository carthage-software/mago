<?php

declare(strict_types=1);

class InhStaticParent
{
    public static function create(): int
    {
        return 1;
    }
}

class InhStaticChild extends InhStaticParent
{
}

echo InhStaticChild::create();
