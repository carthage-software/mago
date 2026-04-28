<?php

declare(strict_types=1);

abstract class InhAbsClassMix
{
    public function helper(): int
    {
        return 1;
    }

    abstract public function action(): int;
}

class InhAbsClassMixChild extends InhAbsClassMix
{
    public function action(): int
    {
        return $this->helper();
    }
}

(new InhAbsClassMixChild())->action();
