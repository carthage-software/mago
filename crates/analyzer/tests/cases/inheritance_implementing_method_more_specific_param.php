<?php

declare(strict_types=1);

interface InhSpecParamIface
{
    public function feed(mixed $value): void;
}

class InhSpecParamImpl implements InhSpecParamIface
{
    /** @mago-expect analysis:incompatible-parameter-type */
    public function feed(int $value): void
    {
    }
}
