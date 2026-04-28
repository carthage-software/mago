<?php

declare(strict_types=1);

interface InhVisN1Iface
{
    public function step(): void;
}

class InhVisN1Impl implements InhVisN1Iface
{
    /** @mago-expect analysis:incompatible-visibility */
    protected function step(): void
    {
    }
}
