<?php

declare(strict_types=1);

interface InhStaticInstIface
{
    public static function build(): self;
}

class InhStaticInstImpl implements InhStaticInstIface
{
    /** @mago-expect analysis:incompatible-static-modifier */
    public function build(): self
    {
        return $this;
    }
}
