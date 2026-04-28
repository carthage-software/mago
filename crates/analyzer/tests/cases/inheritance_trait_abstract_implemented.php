<?php

declare(strict_types=1);

trait InhTAbsImplTrait
{
    abstract public function getValue(): int;

    public function doubled(): int
    {
        return $this->getValue() * 2;
    }
}

class InhTAbsImplUser
{
    use InhTAbsImplTrait;

    public function getValue(): int
    {
        return 21;
    }
}

echo (new InhTAbsImplUser())->doubled();
