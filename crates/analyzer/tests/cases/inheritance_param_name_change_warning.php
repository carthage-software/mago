<?php

declare(strict_types=1);

interface InhPNameIface
{
    public function call(int $alpha): void;
}

class InhPNameImpl implements InhPNameIface
{
    /** @mago-expect analysis:incompatible-parameter-name */
    public function call(int $beta): void
    {
    }
}
