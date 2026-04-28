<?php

declare(strict_types=1);

abstract class InhAbsOkParent
{
    abstract public function action(): int;
}

class InhAbsOkChild extends InhAbsOkParent
{
    public function action(): int
    {
        return 1;
    }
}

echo (new InhAbsOkChild())->action();
