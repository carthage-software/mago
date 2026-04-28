<?php

declare(strict_types=1);

interface InhFewIface
{
    public function run(int $a, int $b): void;
}

class InhFewImpl implements InhFewIface
{
    /** @mago-expect analysis:incompatible-parameter-count */
    public function run(int $a): void
    {
    }
}
