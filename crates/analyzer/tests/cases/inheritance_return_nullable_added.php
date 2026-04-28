<?php

declare(strict_types=1);

interface InhNNIface
{
    public function load(): int;
}

class InhNNImpl implements InhNNIface
{
    /** @mago-expect analysis:incompatible-return-type */
    public function load(): null|int
    {
        return null;
    }
}
