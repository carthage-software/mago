<?php

declare(strict_types=1);

interface InhIntIface
{
    public function exec(): int;
}

class InhIntImpl implements InhIntIface
{
    public function exec(): void {}
}
