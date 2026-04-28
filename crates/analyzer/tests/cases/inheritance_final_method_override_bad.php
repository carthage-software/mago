<?php

declare(strict_types=1);

class InhFinalMethodBase
{
    final public function locked(): void
    {
    }
}

class InhFinalMethodChild extends InhFinalMethodBase
{
    /** @mago-expect analysis:override-final-method */
    public function locked(): void
    {
    }
}
