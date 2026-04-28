<?php

declare(strict_types=1);

abstract class InhVisBase
{
    abstract protected function step(): void;
}

class InhVisChild extends InhVisBase
{
    public function step(): void
    {
    }
}

(new InhVisChild())->step();
