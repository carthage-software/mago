<?php

declare(strict_types=1);

class InhReadonlyParent
{
    public function __construct(
        public readonly int $x,
    ) {}
}

class InhReadonlyChild extends InhReadonlyParent {}

$o = new InhReadonlyChild(1);
$o->x = 2;
