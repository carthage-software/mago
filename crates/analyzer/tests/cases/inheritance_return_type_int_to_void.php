<?php

declare(strict_types=1);

interface InhIntIface
{
    public function exec(): int;
}

class InhIntImpl implements InhIntIface
{
    /** @mago-expect analysis:incompatible-return-type */
    public function exec(): void
    {
    }
}
