<?php

declare(strict_types=1);

class InhFinalMethodBase
{
    final public function locked(): void {}
}

class InhFinalMethodChild extends InhFinalMethodBase
{
    public function locked(): void {}
}
