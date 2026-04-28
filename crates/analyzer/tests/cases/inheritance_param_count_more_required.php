<?php

declare(strict_types=1);

interface InhPCIface
{
    public function run(int $a): void;
}

class InhPCImpl implements InhPCIface
{
    /** @mago-expect analysis:incompatible-parameter-count */
    public function run(int $a, int $b): void
    {
    }
}
