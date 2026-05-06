<?php

declare(strict_types=1);

class InhParentCompat
{
    public function compute(int $x): int
    {
        return $x;
    }
}

class InhChildCompat extends InhParentCompat
{
    #[\Override]
    public function compute(int $x): int
    {
        return $x * 2;
    }
}

echo (new InhChildCompat())->compute(3);
