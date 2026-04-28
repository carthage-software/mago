<?php

declare(strict_types=1);

trait InhMTraitA
{
    public function a(): int
    {
        return 1;
    }
}

trait InhMTraitB
{
    public function b(): int
    {
        return 2;
    }
}

class InhMultiTraitUser
{
    use InhMTraitA;
    use InhMTraitB;
}

$u = new InhMultiTraitUser();
echo $u->a() + $u->b();
