<?php

declare(strict_types=1);

interface InhDropReqIface
{
    public function call(int $a, int $b): void;
}

class InhDropReqImpl implements InhDropReqIface
{
    /** @mago-expect analysis:incompatible-parameter-count */
    public function call(int $a): void
    {
    }
}
