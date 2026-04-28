<?php

declare(strict_types=1);

interface InhOtoRIface
{
    public function call(int $a = 0): void;
}

class InhOtoRImpl implements InhOtoRIface
{
    /** @mago-expect analysis:incompatible-parameter-count */
    public function call(int $a): void
    {
    }
}
