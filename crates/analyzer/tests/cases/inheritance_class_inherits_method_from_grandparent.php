<?php

declare(strict_types=1);

class InhGrand
{
    public function deep(): int
    {
        return 1;
    }
}

class InhMiddle extends InhGrand
{
}

class InhLeaf extends InhMiddle
{
}

echo (new InhLeaf())->deep();
