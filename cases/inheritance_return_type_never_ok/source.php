<?php

declare(strict_types=1);

interface InhNeverIface
{
    public function fail(): mixed;
}

class InhNeverImpl implements InhNeverIface
{
    public function fail(): never
    {
        exit(1);
    }
}
