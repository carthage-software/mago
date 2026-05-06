<?php

declare(strict_types=1);

interface InhPCManyIface
{
    public function call(int $a): void;
}

class InhPCManyImpl implements InhPCManyIface
{
    public function call(int $a, int $b): void {}
}
