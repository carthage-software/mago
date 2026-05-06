<?php

declare(strict_types=1);

interface InhBaseIface
{
    public function base(): int;
}

interface InhDerivedIface extends InhBaseIface
{
    public function derived(): int;
}

class InhDerivedImpl implements InhDerivedIface
{
    public function base(): int
    {
        return 1;
    }

    public function derived(): int
    {
        return 2;
    }
}

$d = new InhDerivedImpl();
echo $d->base();
echo $d->derived();
