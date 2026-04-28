<?php

declare(strict_types=1);

class InhUnrelP1
{
}

class InhUnrelP2
{
}

interface InhUnrelParamIface
{
    public function feed(InhUnrelP1 $a): void;
}

class InhUnrelParamImpl implements InhUnrelParamIface
{
    /** @mago-expect analysis:incompatible-parameter-type */
    public function feed(InhUnrelP2 $a): void
    {
    }
}
