<?php

declare(strict_types=1);

trait InhTAbsViaT
{
    abstract public function pending(): int;

    public function plus(int $x): int
    {
        return $this->pending() + $x;
    }
}

abstract class InhTAbsViaTBase
{
    use InhTAbsViaT;
}

class InhTAbsViaTChild extends InhTAbsViaTBase
{
    public function pending(): int
    {
        return 1;
    }
}

echo (new InhTAbsViaTChild())->plus(2);
