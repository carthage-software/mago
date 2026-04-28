<?php

declare(strict_types=1);

interface InhVoidIface
{
    public function exec(): void;
}

class InhVoidImpl implements InhVoidIface
{
    /** @mago-expect analysis:incompatible-return-type */
    public function exec(): int
    {
        return 1;
    }
}
