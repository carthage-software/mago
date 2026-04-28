<?php

declare(strict_types=1);

interface InhPCManyIface
{
    public function call(int $a): void;
}

class InhPCManyImpl implements InhPCManyIface
{
    /** @mago-expect analysis:incompatible-parameter-count */
    public function call(int $a, int $b): void
    {
    }
}
