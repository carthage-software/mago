<?php

declare(strict_types=1);

class InhMethDeprParent
{
    /** @deprecated */
    public function old(): void
    {
    }
}

class InhMethDeprChild extends InhMethDeprParent
{
}

/** @mago-expect analysis:deprecated-method */
(new InhMethDeprChild())->old();
