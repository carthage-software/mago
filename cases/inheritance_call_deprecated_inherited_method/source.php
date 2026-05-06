<?php

declare(strict_types=1);

class InhMethDeprParent
{
    /** @deprecated */
    public function old(): void {}
}

class InhMethDeprChild extends InhMethDeprParent {}

(new InhMethDeprChild())->old();
