<?php

declare(strict_types=1);

interface InhFewIface
{
    public function run(int $a, int $b): void;
}

class InhFewImpl implements InhFewIface
{
    public function run(int $a): void {}
}
