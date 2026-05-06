<?php

declare(strict_types=1);

interface InhMixSpecIface
{
    public function get(): mixed;
}

class InhMixSpecImpl implements InhMixSpecIface
{
    public function get(): int
    {
        return 1;
    }
}

(new InhMixSpecImpl())->get();
