<?php

declare(strict_types=1);

trait InhInnerTrait
{
    public function inner(): int
    {
        return 1;
    }
}

trait InhOuterTrait
{
    use InhInnerTrait;

    public function outer(): int
    {
        return $this->inner() + 1;
    }
}

class InhTraitInTraitUser
{
    use InhOuterTrait;
}

$u = new InhTraitInTraitUser();
echo $u->outer();
echo $u->inner();
