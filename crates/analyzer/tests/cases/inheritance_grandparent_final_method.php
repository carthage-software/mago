<?php

declare(strict_types=1);

class InhGFinalGrand
{
    final public function locked(): int
    {
        return 1;
    }
}

class InhGFinalMiddle extends InhGFinalGrand
{
}

class InhGFinalChild extends InhGFinalMiddle
{
    /** @mago-expect analysis:override-final-method */
    public function locked(): int
    {
        return 2;
    }
}
