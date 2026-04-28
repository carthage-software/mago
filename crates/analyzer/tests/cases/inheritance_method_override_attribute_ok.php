<?php

declare(strict_types=1);

class InhOverAttrParent
{
    public function step(): int
    {
        return 1;
    }
}

class InhOverAttrChild extends InhOverAttrParent
{
    #[\Override]
    public function step(): int
    {
        return 2;
    }
}

(new InhOverAttrChild())->step();
