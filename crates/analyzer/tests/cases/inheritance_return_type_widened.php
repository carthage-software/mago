<?php

declare(strict_types=1);

class InhRetParent
{
    public function fetch(): string
    {
        return '';
    }
}

class InhRetChild extends InhRetParent
{
    /** @mago-expect analysis:incompatible-return-type */
    public function fetch(): string|int
    {
        return 0;
    }
}
