<?php

declare(strict_types=1);

interface InhPCOIface
{
    public function run(int $a): void;
}

class InhPCOImpl implements InhPCOIface
{
    public function run(int $a, int $b = 0): void
    {
    }
}
