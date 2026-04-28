<?php

declare(strict_types=1);

class InhPromoParent
{
    public function __construct(public int $a, public int $b)
    {
    }
}

class InhPromoChild extends InhPromoParent
{
    public function __construct(int $a, int $b, public int $c)
    {
        parent::__construct($a, $b);
    }
}

$o = new InhPromoChild(1, 2, 3);
echo $o->a + $o->b + $o->c;
