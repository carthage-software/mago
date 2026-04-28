<?php

declare(strict_types=1);

interface InhSpecMixIface
{
    public function get(): int;
}

class InhSpecMixImpl implements InhSpecMixIface
{
    /** @mago-expect analysis:incompatible-return-type */
    public function get(): mixed
    {
        return 1;
    }
}
