<?php

declare(strict_types=1);

class InhGFinalGrand
{
    final public function locked(): int
    {
        return 1;
    }
}

class InhGFinalMiddle extends InhGFinalGrand {}

class InhGFinalChild extends InhGFinalMiddle
{
    public function locked(): int
    {
        return 2;
    }
}
