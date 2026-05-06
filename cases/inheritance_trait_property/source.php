<?php

declare(strict_types=1);

trait InhTraitProp
{
    public int $count = 0;

    public function bump(): void
    {
        $this->count++;
    }
}

class InhTraitPropUser
{
    use InhTraitProp;
}

$u = new InhTraitPropUser();
$u->bump();
echo $u->count;
