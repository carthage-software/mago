<?php

declare(strict_types=1);

interface InhVisN2Iface
{
    public function step(): void;
}

class InhVisN2Impl implements InhVisN2Iface
{
    /** @mago-expect analysis:incompatible-visibility */
    private function step(): void
    {
    }
}
