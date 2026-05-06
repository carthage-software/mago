<?php

declare(strict_types=1);

class InhInstStaticParent
{
    public function step(): int
    {
        return 1;
    }
}

class InhInstStaticChild extends InhInstStaticParent
{
    public static function step(): int
    {
        return 2;
    }
}
